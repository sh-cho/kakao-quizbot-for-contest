use axum::{Json, Router};
use axum::extract::State;
use axum::routing::post;
use kakao_rs::prelude::{SimpleText, Template};
use tracing::debug;

use game::state::Command;

use crate::{Error, game, Result};
use crate::game::state::GameManager;
use crate::web::model::BotRequest;
use crate::web::model::ChatIdType::BotGroupKey;

pub fn routes(
    gm: GameManager,
) -> Router {
    Router::new()
        .route("/", post(bot_request))
        .with_state(gm)
}

async fn bot_request(
    State(gm): State<GameManager>,
    Json(payload): Json<BotRequest>,
) -> Result<Json<Template>> {
    debug!("{:<12} - bot_request", "HANDLER");

    let user_id = payload.user_request.user.id;
    let chat_id = payload.user_request.chat.id;
    if payload.user_request.chat.type_ != BotGroupKey {
        return Err(Error::ChatTypeNotSupported(payload.user_request.chat.type_));
    }

    let utterance = payload.user_request.utterance;
    let command = Command::from_utterance(&utterance)
        .ok_or(Error::GameCommandParseFail(utterance))?;

    let mut response = Template::new();
    match command {
        Command::Start => {
            let game = gm.start_game(chat_id).await?;
            response.add_output(SimpleText::new(game.current_quiz.info_before()).build());
        }
        Command::Stop => {
            gm.stop_game(chat_id).await?;
            response.add_output(SimpleText::new("퀴즈게임이 종료되었습니다.").build());
        }
        Command::Answer(answer) => {
            let result = gm.try_answer(user_id, chat_id, answer).await?;

            match result {
                game::state::AnswerResult::Correct { user_id, score } => {
                    response.add_output(SimpleText::new(format!("{} 정답! (점수: {})", user_id, score)).build());
                }
                game::state::AnswerResult::Wrong => {
                    // TODO: no-op
                    response.add_output(SimpleText::new("땡").build());
                }
            }
        
            // TODO: next quiz
        }
        Command::Ranking => {
            let (user_rank, chat_rank) = gm.get_ranking(&user_id, &chat_id).await?;
            response.add_output(SimpleText::new(format!("당신의 순위: {}등\n이 방의 순위: {}등", user_rank, chat_rank)).build());
        }
    }

    Ok(Json(response))
}
