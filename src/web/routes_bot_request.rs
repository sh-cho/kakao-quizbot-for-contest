use axum::{Json, Router};
use axum::extract::State;
use axum::routing::post;
use kakao_rs::prelude::{SimpleText, Template};
use tracing::debug;

use crate::web::model::Command;

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

pub async fn bot_request(
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
        .ok_or(Error::GameCommandParseFail("🗒️ 명령어 목록
- 시작
- 중지
- 정답 OOO
- 랭킹(🚧)"))?;

    let mut response = Template::new();
    match command {
        Command::Start => {
            let game = gm.start_game(chat_id).await?;
            response.add_output(SimpleText::new(game.current_quiz.info_before(game.current_round)).build());
        }
        Command::Stop => {
            gm.stop_game(chat_id).await?;
            response.add_output(SimpleText::new("🔴 퀴즈게임이 종료되었습니다.").build());
        }
        Command::Answer(answer) => {
            let result = gm.try_answer_inmemory(&user_id, &chat_id, &answer).await?;
            match result {
                game::state::AnswerResult::Correct {
                    user_id,
                    score,
                    current_quiz,
                    next_quiz,
                    current_round
                } => {
                    // TODO: hash -> nickname?
                    let mut result_text = format!("👏 {:.6} 정답! (점수: {})", user_id, score);
                    if let Some(comment) = current_quiz.comment.clone() {
                        result_text.push_str(format!("\n{}", comment).as_str());
                    }

                    response.add_output(SimpleText::new(result_text).build());

                    if current_round > game::state::MAX_ROUNDS {
                        response.add_output(SimpleText::new("✅ 다 풀었습니다 :)").build());
                        gm.stop_game(chat_id).await?;
                    } else {
                        response.add_output(SimpleText::new(next_quiz.info_before(current_round)).build());
                    }
                }
                game::state::AnswerResult::Wrong => {
                    // no-op
                    // response.add_output(SimpleText::new("[DEBUG] 땡").build());
                }
            }
        }
        Command::Ranking => {
            // let (user_rank, chat_rank) = gm.get_ranking(&user_id, &chat_id).await?;
            // response.add_output(SimpleText::new(format!("당신의 순위: {}등\n이 방의 순위: {}등", user_rank, chat_rank)).build());
            
            response.add_output(SimpleText::new("🚧 공사중").build());
        }
    }

    Ok(Json(response))
}
