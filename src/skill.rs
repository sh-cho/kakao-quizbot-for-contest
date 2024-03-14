use std::collections::HashMap;
use kakao_rs::prelude::Template;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TemplateWithExtra {
    pub template: Template,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Extra>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Extra {
    pub mentions: HashMap<String, Mention>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    #[serde(rename = "type")]
    pub mention_type: String,  // botUserKey
    pub id: String,
}
