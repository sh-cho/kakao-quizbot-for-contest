//! Redis key structure
//! ---
//!
//! ## scores
//! - key: `group:{group_key}` / value: `{score}`
//! - key: `user:{user_id}`    / value: `{score}`
//!
//! ## ranking
//! - key: user_scores (ZSET)
//! - key: group_scores (ZSET)
//!
//! ZADD
//! ZRANK

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use redis::AsyncCommands;
use tokio::sync::RwLock;

use crate::{Error, Result};
use crate::conn::RedisConnectionPool;
use crate::game::db::quiz_db;
use crate::game::model::Quiz;
use crate::game::state::AnswerResult::Correct;

pub type GroupKey = String;

const REDIS_USER_SCORES_KEY: &str = "user_scores";
const REDIS_GROUP_SCORES_KEY: &str = "group_scores";

// XXX: is this right way?
#[derive(Clone)]
pub struct GameManager {
    games: Arc<RwLock<HashMap<GroupKey, Mutex<Game>>>>,
    pool: RedisConnectionPool,
}

impl GameManager {
    pub fn new(pool: RedisConnectionPool) -> Result<Self> {
        Ok(Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            pool,
        })
    }

    pub async fn start_game(&self, group_key: GroupKey) -> Result<Game> {
        let mut games = self.games.write().await;
        let game = Game::new(group_key.clone());

        games.insert(group_key.clone(), Mutex::new(game.clone()))
            .ok_or(Error::GameAlreadyStarted(group_key))?;

        Ok(game)
    }

    pub async fn stop_game(&self, group_key: GroupKey) -> Result<()> {
        let mut games = self.games.write().await;
        games.remove(&group_key)
            .ok_or(Error::GameNotFound(group_key))?;

        Ok(())
    }

    // TODO: race cond?
    pub async fn try_answer(&self, user_id: String, group_key: GroupKey, answer: String) -> Result<AnswerResult> {
        let games = self.games.read().await;
        let game = games.get(&group_key)
            .ok_or(Error::GameNotFound(group_key.clone()))?;

        let mut game = game.lock().unwrap();
        if game.current_quiz.answer != answer {
            return Ok(AnswerResult::Wrong);
        }

        game.current_round += 1;
        game.current_quiz = quiz_db().get_random_quiz();

        // +1
        // TODO: use pipeline
        let mut conn = self.pool.get()
            .await
            .map_err(|_| Error::RedisConnectionGetFail)?;

        // scores
        // XXX: `user_id` is different in different chat, so...
        let redis_key = format!("user:{}", user_id.clone());
        let _: () = conn.incr(redis_key.clone(), 1)
            .await
            .map_err(|_| Error::RedisCommandFail(redis_key.clone()))?;

        let redis_key = format!("group:{}", group_key);
        let _: () = conn.incr(redis_key.clone(), 1)
            .await
            .map_err(|_| Error::RedisCommandFail(redis_key.clone()))?;
        
        // hget
        let score: i64 = conn.get(redis_key.clone()).await
            .map_err(|_| Error::RedisCommandFail(redis_key.clone()))?;

        // ranks
        conn.zadd(REDIS_USER_SCORES_KEY, user_id.clone(), score).await
            .map_err(|_| Error::RedisCommandFail(REDIS_USER_SCORES_KEY.to_string()))?;
        conn.zadd(REDIS_GROUP_SCORES_KEY, group_key, score).await
            .map_err(|_| Error::RedisCommandFail(REDIS_GROUP_SCORES_KEY.to_string()))?;

        Ok(Correct {
            user_id,
            // score: score as u32,
            score: 1,
        })
    }

    // Return user ranking and group ranking, using ZRANK
    // TODO
    pub async fn get_ranking(&self, user_id: &str, group_key: &str) -> Result<(i32, i32)> {
        let mut conn = self.pool.get()
            .await
            .map_err(|_| Error::RedisConnectionGetFail)?;

        let user_rank = conn.zrank(REDIS_USER_SCORES_KEY, user_id).await
            .map_err(|_| Error::RedisCommandFail(REDIS_USER_SCORES_KEY.to_string()))?;
        let group_rank = conn.zrank(REDIS_GROUP_SCORES_KEY, group_key).await
            .map_err(|_| Error::RedisCommandFail(REDIS_GROUP_SCORES_KEY.to_string()))?;

        Ok((user_rank, group_rank))
    }
}

// max time per round: 60 seconds
// max rounds: 10
// 굳이 남겨놓지 않는게 좋을듯.
const MAX_ROUNDS: u8 = 10;

#[derive(Clone)]
pub struct Game {
    group_key: GroupKey,
    current_round: u8,
    pub current_quiz: &'static Quiz,
}

impl Game {
    pub fn new(group_key: GroupKey) -> Self {
        Self {
            group_key,
            current_round: 0,
            current_quiz: quiz_db().get_random_quiz(),
        }
    }
}

pub enum AnswerResult {
    Correct {
        user_id: String,
        // NOTE: redis integer is i64, but for now it's enough to use u32
        score: u32,
    },
    Wrong,
}

pub enum Command {
    Start,
    Stop,
    Answer(String),
    Ranking,
}

impl Command {
    pub fn from_utterance(utterance: &str) -> Option<Command> {
        let utterance = utterance.trim();
        let command = utterance.splitn(2, ' ').next()?;

        match command {
            "시작" => Some(Command::Start),
            "중지" => Some(Command::Stop),
            // TODO: "정답" 명령어를 사용하지 않고, 바로 답 입력하도록
            "정답" => {
                let answer = utterance.splitn(2, ' ').nth(1)?;
                Some(Command::Answer(answer.to_string()))
            }
            "랭킹" => Some(Command::Ranking),
            _ => None,
        }
    }
}
