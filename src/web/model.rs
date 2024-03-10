use serde::{Deserialize, Serialize};

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
