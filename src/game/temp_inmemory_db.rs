//! static in-memory hashmap for the game state

use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::game::state::GroupKey;

lazy_static! {
    // 1. score by group (in redis, (key: group:{group_key}, value: score))
    pub static ref SCORES_BY_GROUP: Mutex<HashMap<GroupKey, u32>> = Mutex::new(HashMap::new());

    // 2. score by user (in redis, (key: user:{user_id}, value: score))
    pub static ref SCORES_BY_USER: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());

    // TODO: Ranking
}
