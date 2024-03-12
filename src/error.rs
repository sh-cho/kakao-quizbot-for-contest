use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use kakao_rs::prelude::{SimpleText, Template};
use serde::Serialize;
use serde_with::serde_as;
use crate::game::state::GroupKey;
use crate::web::model::ChatIdType;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // -- Sys
    ConfigMissingEnv(&'static str),
    RedisConnectionGetFail,
    RedisCommandFail(String), // key

    // -- Bot
    AuthFail,
    ChatTypeNotSupported(ChatIdType),
    
    // -- Game
    GameCommandParseFail(&'static str),  // utterance? 보다는 그냥 에러메시지
    GameNotFound(GroupKey),
    GameAlreadyStarted(GroupKey),
    GameAlreadyFinished(GroupKey),
    GameInvalidCategoryName,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        // create placeholder axum response
        // let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the repsonse
        // response.extensions_mut().insert(self);
        
        // Json<Template>
        let mut template = Template::new();
        
        match self {
            Error::GameCommandParseFail(help_message) => {
                template.add_output(SimpleText::new(help_message).build());
            }
            _ => {
                template.add_output(SimpleText::new(format!("err: {self:?}").as_str()).build());
            }
        }
        
        // for now, just return 200 with template body
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&template).unwrap())
            .unwrap()
            .into_response()
    }
}
