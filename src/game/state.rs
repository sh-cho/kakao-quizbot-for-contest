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
use crate::game::db::{QUIZ_CATEGORIES, quiz_db};
use crate::game::model::Quiz;
use crate::game::temp_inmemory_db::{SCORES_BY_GROUP, SCORES_BY_USER};

pub type GroupKey = String;

const REDIS_USER_SCORES_KEY: &str = "user_scores";
const REDIS_GROUP_SCORES_KEY: &str = "group_scores";

// XXX: is this right way?
#[derive(Clone)]
pub struct GameManager {
    games: Arc<RwLock<HashMap<GroupKey, Mutex<Game>>>>,
    
    // pool: RedisConnectionPool,  // to-be-done
    
    // for now, just use static
}

impl GameManager {
    pub fn new(pool: RedisConnectionPool) -> Result<Self> {
        Ok(Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            // pool,
        })
    }

    pub async fn start_game(&self, group_key: GroupKey, category_name: Option<String>) -> Result<Game> {
        let mut games = self.games.write().await;
        let game = Game::new(group_key.clone(), category_name);

        let prev = games.insert(group_key.clone(), Mutex::new(game.clone()));
        match prev {
            Some(_) => Err(Error::GameAlreadyStarted(group_key)),
            None => Ok(game),
        }
    }

    // fn is_valid_category(&self, category_name: &String) -> bool {
    //     QUIZ_CATEGORIES.contains(category_name)
    // }

    pub async fn stop_game(&self, group_key: GroupKey) -> Result<()> {
        let mut games = self.games.write().await;
        games.remove(&group_key)
            .ok_or(Error::GameNotFound(group_key))?;

        Ok(())
    }

    pub async fn try_answer_inmemory(&self, user_id: &str, group_key: &GroupKey, answer: &str) -> Result<AnswerResult> {
        let games = self.games.read().await;
        let game = games.get(group_key)
            .ok_or(Error::GameNotFound(group_key.clone()))?;
        
        let mut game = game.lock().unwrap();
        if game.current_quiz.answer != answer {
            return Ok(AnswerResult::Wrong);
        }
        
        // scores
        let mut scores_by_user = SCORES_BY_USER.lock().unwrap();
        let mut scores_by_group = SCORES_BY_GROUP.lock().unwrap();
        
        *scores_by_user.entry(user_id.to_string()).or_insert(0) += 1;
        *scores_by_group.entry(group_key.clone()).or_insert(0) += 1;

        let current_quiz = game.current_quiz;
        
        game.current_round += 1;
        // game.current_quiz = quiz_db().get_any_random_quiz();

        // category에 따라. 없는 카테고리면 랜덤하게
        // TODO: 한쪽으로 정리. 시작할 떄?
        if let Some(category) = &game.selected_category {
            game.current_quiz = quiz_db().get_random_quiz_by_category(category)
                .unwrap_or(quiz_db().get_any_random_quiz());
        } else {
            game.current_quiz = quiz_db().get_any_random_quiz();
        }

        // TODO: rank
        
        Ok(AnswerResult::Correct {
            user_id: user_id.to_string(),
            score: *scores_by_user.get(user_id).unwrap(),
            current_quiz,
            next_quiz: game.current_quiz,
            current_round: game.current_round,
        })
    }
    
    // // TODO: race cond?
    // pub async fn try_answer_with_redis(&self, user_id: String, group_key: GroupKey, answer: String) -> Result<AnswerResult> {
    //     let games = self.games.read().await;
    //     let game = games.get(&group_key)
    //         .ok_or(Error::GameNotFound(group_key.clone()))?;
    //
    //     let mut game = game.lock().unwrap();
    //     if game.current_quiz.answer != answer {
    //         return Ok(AnswerResult::Wrong);
    //     }
    //
    //     game.current_round += 1;
    //     game.current_quiz = quiz_db().get_random_quiz();
    //
    //     // +1
    //     // TODO: use pipeline
    //     let mut conn = self.pool.get()
    //         .await
    //         .map_err(|_| Error::RedisConnectionGetFail)?;
    //
    //     // let conn = self.pool.
    //     // let pool = self.pool.clone();
    //     // let conn = pool.get();
    //     //
    //     // conn.await.unwrap();
    //
    //     // scores
    //     // XXX: `user_id` is different in different chat, so...
    //     let redis_key = format!("user:{}", user_id.clone());
    //     let _: () = conn.incr(redis_key.clone(), 1)
    //         .await
    //         .map_err(|_| Error::RedisCommandFail(redis_key.clone()))?;
    //
    //     let redis_key = format!("group:{}", group_key);
    //     let _: () = conn.incr(redis_key.clone(), 1)
    //         .await
    //         .map_err(|_| Error::RedisCommandFail(redis_key.clone()))?;
    //
    //     // hget
    //     let score: i64 = conn.get(redis_key.clone()).await
    //         .map_err(|_| Error::RedisCommandFail(redis_key.clone()))?;
    //
    //     // ranks
    //     conn.zadd(REDIS_USER_SCORES_KEY, user_id.clone(), score).await
    //         .map_err(|_| Error::RedisCommandFail(REDIS_USER_SCORES_KEY.to_string()))?;
    //     conn.zadd(REDIS_GROUP_SCORES_KEY, group_key, score).await
    //         .map_err(|_| Error::RedisCommandFail(REDIS_GROUP_SCORES_KEY.to_string()))?;
    //
    //     // Ok(Correct {
    //     //     user_id,
    //     //     // score: score as u32,
    //     //     score: 1,
    //     // })
    //
    //     todo!()
    // }

    // Return user ranking and group ranking, using ZRANK
    // TODO
    pub async fn get_ranking(&self, user_id: &str, group_key: &str) -> Result<(i32, i32)> {
        // let mut conn = self.pool.get()
        //     .await
        //     .map_err(|_| Error::RedisConnectionGetFail)?;
        // 
        // let user_rank = conn.zrank(REDIS_USER_SCORES_KEY, user_id).await
        //     .map_err(|_| Error::RedisCommandFail(REDIS_USER_SCORES_KEY.to_string()))?;
        // let group_rank = conn.zrank(REDIS_GROUP_SCORES_KEY, group_key).await
        //     .map_err(|_| Error::RedisCommandFail(REDIS_GROUP_SCORES_KEY.to_string()))?;
        // 
        // Ok((user_rank, group_rank))
        
        Ok((123, 123))
    }
}

// max time per round: 60 seconds
// max rounds: 10
// 굳이 남겨놓지 않는게 좋을듯.
// pub const MAX_ROUNDS: u8 = 10;
pub const MAX_ROUNDS: u8 = 3;

#[derive(Clone)]
pub struct Game {
    group_key: GroupKey,
    pub current_round: u8,
    pub current_quiz: &'static Quiz,
    pub selected_category: Option<String>,  // 없으면 all random
}

impl Game {
    pub fn new(group_key: GroupKey, selected_category: Option<String>) -> Self {
        Self {
            group_key,
            current_round: 1,
            current_quiz: quiz_db().get_any_random_quiz(),
            selected_category,
        }
    }
}

pub enum AnswerResult {
    Correct {
        user_id: String,
        // NOTE: redis integer is i64, but for now it's enough to use u32
        score: u32,
        current_quiz: &'static Quiz,
        next_quiz: &'static Quiz,
        current_round: u8,
    },
    Wrong,
}
