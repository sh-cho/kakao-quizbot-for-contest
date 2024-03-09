use axum::{Json, Router};
use axum::routing::post;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use kakao_rs::prelude::Template;
use crate::{Error, game, RedisConnection};
use crate::web::model::BotRequest;

pub fn routes(pool: Pool<RedisConnectionManager>) -> Router {
    Router::new()
        .route("/", post(bot_request))
        .with_state(pool)
}

async fn bot_request(
    RedisConnection(mut conn): RedisConnection,
    Json(payload): Json<BotRequest>,
) -> crate::Result<Json<Template>> {
    let user_id = payload.user_request.user.id;
    let chat_id = payload.user_request.chat.id;
    let utterance = payload.user_request.utterance;

    let command = game::Command::from_utterance(&utterance);
    if command.is_none() {
        return Err(Error::GameCommandParseFail);
    }
    
    //

    todo!()
}
