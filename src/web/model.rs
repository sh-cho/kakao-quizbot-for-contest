use serde::{Deserialize, Serialize};
use crate::game::db::QUIZ_CATEGORIES;

/// bot proxy -> skill server payload
/// skipped unused fields
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotRequest {
    pub user_request: UserRequest,
}

#[derive(Debug, Deserialize)]
pub struct UserRequest {
    pub user: User,
    pub chat: Chat,
    pub utterance: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: String,

    #[serde(rename = "type")]
    pub type_: ChatIdType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChatIdType {
    BotGroupKey,
    ChatId,     // Unused
}

pub enum Command {
    Start(Option<String>),
    Stop,
    Answer(String),
    Ranking,
}

impl Command {
    pub fn from_utterance(utterance: &str) -> Option<Command> {
        let utterance = utterance.trim();
        let command = utterance.splitn(2, ' ').next()?;

        match command {
            "시작" => {
                // 카테고리는 있을 수도 있고 없을 수도 있다
                let category = utterance.splitn(2, ' ')
                    .nth(1)
                    .map(|s| s.to_string());

                // 유효하지 않은 카테고리
                if let Some(ref category) = category {
                    if !QUIZ_CATEGORIES.contains(category.as_str()) {
                        return None;
                    }
                }

                Some(Command::Start(category))
            }
            "중지" | "중단" | "정지" | "종료" | "그만" | "멈춰" => Some(Command::Stop),
            // TODO: "정답" 명령어를 사용하지 않고, 바로 답 입력하도록 ?
            "정답" => {
                let answer = utterance.splitn(2, ' ').nth(1)?;
                Some(Command::Answer(answer.to_string()))
            }
            "랭킹" | "순위" => Some(Command::Ranking),
            _ => None,
        }
    }
}
