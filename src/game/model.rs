use serde::Deserialize;
use crate::game;

#[derive(Debug, Deserialize)]
pub struct Quiz {
    pub category: String,
    question: String,
    pub answer: String,
    pub comment: Option<String>,
}

impl Quiz {
    pub fn info_before(&self, current_round: u8) -> String {
        format!("[{}/{}] ({})\n{}", current_round, game::state::MAX_ROUNDS, self.category, self.question)
    }
    
    // pub fn info_after(&self) -> String {
    //     format!("정답: {}\n{}", self.answer, self.comment.as_deref().unwrap_or_default())
    // }
}
