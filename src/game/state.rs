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
use std::sync::Arc;
use std::time::Duration;

use redis::AsyncCommands;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::time::timeout;
use tracing::debug;

use crate::{Error, Result};
use crate::conn::RedisConnectionPool;
use crate::game::db::{flag_quiz_db, QUIZ_CATEGORIES, quiz_db, QuizType};
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

    pub http_client: reqwest::Client,
}

impl GameManager {
    pub fn new(pool: RedisConnectionPool) -> Result<Self> {
        Ok(Self {
            games: Arc::new(RwLock::new(HashMap::new())),
            // pool,
            http_client: reqwest::Client::new(),
        })
    }

    pub async fn start_game(&self, group_key: GroupKey, category_name: Option<String>, is_flag_quiz: bool) -> Result<Game> {
        debug!("{:<12} - start_game, group_key: {}, category_name: {:?}, is_flag_quiz: {}", "GAME", group_key, category_name, is_flag_quiz);
        let mut games = self.games.write().await;
        let game = Game::new(group_key.clone(), category_name, is_flag_quiz);

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

        let mut game = game.lock().await;
        let is_correct = match &game.current_quiz {  // 으아악
            QuizType::Simple(quiz) => quiz.is_correct_answer(answer),
            QuizType::Flag(flag_quiz) => flag_quiz.is_correct_answer(answer),
        };
        if !is_correct {
            return Ok(AnswerResult::Wrong);
        }

        // scores
        let mut scores_by_user = SCORES_BY_USER.lock().unwrap();
        let mut scores_by_group = SCORES_BY_GROUP.lock().unwrap();

        *scores_by_user.entry(user_id.to_string()).or_insert(0) += 1;
        *scores_by_group.entry(group_key.clone()).or_insert(0) += 1;

        let current_quiz = game.current_quiz.clone();

        game.current_round += 1;
        // category에 따라. 없는 카테고리면 랜덤하게
        // TODO: 한쪽으로 정리. 시작할 떄?
        if let Some(category) = &game.selected_category {
            // game.current_quiz = quiz_db().get_random_quiz_by_category(category)
            //     .unwrap_or(quiz_db().get_any_random_quiz());

            // 국기인 경우에는 국기 문제만
            if category == "국기" {
                game.current_quiz = QuizType::Flag(flag_quiz_db().get_random_flag_quiz().clone());
            } else {
                let next_quiz = quiz_db().get_random_quiz_by_category(category)
                    .unwrap_or(quiz_db().get_any_random_quiz());

                game.current_quiz = QuizType::Simple(next_quiz.clone());
            }
        } else {
            game.current_quiz = QuizType::Simple(quiz_db().get_any_random_quiz().clone());
        }

        // TODO: rank

        Ok(AnswerResult::Correct {
            user_id: user_id.to_string(),
            score: *scores_by_user.get(user_id).unwrap(),
            current_quiz,
            next_quiz: game.current_quiz.clone(),
            current_round: game.current_round,
        })
    }

    // region: redis (TODO)
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
    // endregion: redis

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

    pub async fn submit_answer(&self, game_id: String) {
        let games = self.games.write().await;
        let game = games.get(&game_id).unwrap().lock().await;

        // Notify the waiting task.
        game.answer_submitted.notify_one();
    }

    pub async fn wait_for_answer(&self, game_id: String) -> std::result::Result<(), tokio::time::error::Elapsed> {
        let games = self.games.read().await;
        let game = games.get(&game_id).unwrap().lock().await;

        // Wait for the answer to be submitted or for the timeout to expire.
        timeout(
            Duration::from_secs(MAX_TIMEOUT_SECONDS),
            game.answer_submitted.notified(),
        ).await?;

        Ok(())
    }

    // 현재 라운드의 답을 반환하고, 다음 라운드로 넘어가기 (아무도 라운드 못 풀었을 때용)
    // return: (current_quiz, next_quiz, next_round)
    pub async fn get_current_quiz_and_move_next(&self, group_key: GroupKey) -> Result<(QuizType, QuizType, u8)> {
        let games = self.games.read().await;
        let game = games.get(group_key.as_str())
            .ok_or(Error::GameNotFound(group_key.clone()))?;


        let mut game = game.lock().await;
        let current_quiz = game.current_quiz.clone();

        // game.current_round += 1;
        // game.current_quiz = quiz_db().get_any_random_quiz().clone();
        // 
        // Ok((current_quiz, &game.current_quiz, game.current_round))

        // TODO: duplicated
        game.current_round += 1;
        // category에 따라. 없는 카테고리면 랜덤하게
        // TODO: 한쪽으로 정리. 시작할 떄?
        if let Some(category) = &game.selected_category {
            // game.current_quiz = quiz_db().get_random_quiz_by_category(category)
            //     .unwrap_or(quiz_db().get_any_random_quiz());

            // 국기인 경우에는 국기 문제만
            if category == "국기" {
                game.current_quiz = QuizType::Flag(flag_quiz_db().get_random_flag_quiz().clone());
            } else {
                let next_quiz = quiz_db().get_random_quiz_by_category(category)
                    .unwrap_or(quiz_db().get_any_random_quiz());

                game.current_quiz = QuizType::Simple(next_quiz.clone());
            }
        } else {
            game.current_quiz = QuizType::Simple(quiz_db().get_any_random_quiz().clone());
        }

        Ok((current_quiz, game.current_quiz.clone(), game.current_round))
    }

    // 게임이 존재하는지 여부 확인
    pub async fn is_game_exists(&self, group_key: &GroupKey) -> bool {
        let games = self.games.read().await;
        games.contains_key(group_key)
    }
}

// max time per round: 60 seconds
// max rounds: 10
// 굳이 남겨놓지 않는게 좋을듯.
// pub const MAX_ROUNDS: u8 = 10;
pub const MAX_ROUNDS: u8 = 3;
pub const MAX_TIMEOUT_SECONDS: u64 = 60;

#[derive(Clone)]
pub struct Game {
    group_key: GroupKey,
    pub current_round: u8,
    // pub current_quiz: &'static Quiz,
    pub current_quiz: QuizType,
    pub selected_category: Option<String>,
    // 없으면 all random

    answer_submitted: Arc<Notify>,  // 누군가 정답을 맞췄을 떄
}

impl Game {
    pub fn new(group_key: GroupKey, selected_category: Option<String>, is_flag_quiz: bool) -> Self {
        Self {
            group_key,
            current_round: 1,
            current_quiz: if is_flag_quiz {
                QuizType::Flag(flag_quiz_db().get_random_flag_quiz().clone())
            } else {
                QuizType::Simple(quiz_db().get_any_random_quiz().clone())
            },
            selected_category,
            answer_submitted: Arc::new(Notify::new()),
        }
    }
}

pub enum AnswerResult {
    Correct {
        user_id: String,
        // NOTE: redis integer is i64, but for now it's enough to use u32
        score: u32,
        // current_quiz: &'static Quiz,
        current_quiz: QuizType,
        // next_quiz: &'static Quiz,
        next_quiz: QuizType,
        current_round: u8,
    },
    Wrong,
}
