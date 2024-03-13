use serde::Deserialize;

use crate::game;
use crate::game::db::{FLAG_IMAGE_CDN_PATH, FLAG_IMAGE_EXT};

#[derive(Debug, Clone, Deserialize)]
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
    
    pub fn is_correct_answer(&self, answer: &str) -> bool {
        self.answer == answer
    }

    // pub fn info_after(&self) -> String {
    //     format!("정답: {}\n{}", self.answer, self.comment.as_deref().unwrap_or_default())
    // }
}

// TODO: 텍스트 퀴즈랑 동일 trait으로 묶기
// extension은 현재는 전부 png
#[derive(Debug, Clone, Deserialize)]
pub struct FlagQuiz {
    // ex) "kr"
    pub country_code_alpha_2: String,

    // ex) "대한민국"
    #[serde(rename = "country_name")]
    pub answer: String,
}

impl FlagQuiz {
    pub fn is_correct_answer(&self, answer: &str) -> bool {
        self.answer == answer
    }
    
    pub fn info_before(&self, current_round: u8) -> String {
        format!("[{}/{}] 이 국기는 어느 나라의 국기일까요?", current_round, game::state::MAX_ROUNDS)
    }

    // pub fn title(&self, current_round: u8) -> String {
    //     format!("[{}/{}]", current_round, game::state::MAX_ROUNDS)
    // }
    
    pub fn image_url(&self) -> String {
        format!("{}/{}.{}", FLAG_IMAGE_CDN_PATH, self.country_code_alpha_2, FLAG_IMAGE_EXT)
    }
}


pub trait QuizTrait {
    // TODO: info ?
    
    fn get_answer(&self) -> &str;
}

impl QuizTrait for Quiz {
    fn get_answer(&self) -> &str {
        &self.answer
    }
}

impl QuizTrait for FlagQuiz {
    fn get_answer(&self) -> &str {
        &self.answer
    }
}
