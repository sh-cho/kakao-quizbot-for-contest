use axum::{Json, Router};
use axum::extract::State;
use axum::routing::post;
use kakao_rs::prelude::{BasicCard, SimpleImage, SimpleText, Template};
use tracing::debug;

use crate::{Error, game, Result};
use crate::config::config;
use crate::game::db::QuizType;
use crate::game::model::QuizTrait;
use crate::game::state::GameManager;
use crate::web::model::BotRequest;
use crate::web::model::ChatIdType::BotGroupKey;
use crate::web::model::Command;

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
        .ok_or(Error::GameCommandParseFail(r#"🗒️ 명령어 목록
- 시작 [카테고리]: 카테고리를 입력하지 않으면 전체 문제를 대상으로 출제됩니다.
  (사용 가능 카테고리: 국기(추천), 상식, 고사성어)
- 중지
- 정답 OOO
- 랭킹(🚧)"#))?;

    let mut response = Template::new();
    match command {
        Command::Start(category) => {
            let is_flag_quiz = category.as_deref() == Some("국기");
            let game = gm.start_game(chat_id.clone(), category, is_flag_quiz).await?;

            // todo: extract
            match &game.current_quiz {
                QuizType::Simple(quiz) => {
                    response.add_output(SimpleText::new(quiz.info_before(game.current_round)).build());
                }
                QuizType::Flag(quiz) => {
                    // BasicCard -> 이미지 비율이 제한적이라 안쓰는걸루
                    // response.add_output(
                    //     BasicCard::new()
                    //         .set_title(quiz.title(game.current_round))
                    //         .set_description("이 국기는 어느 나라의 국기일까요?")
                    //         .set_thumbnail(quiz.image_url())
                    //     .build()
                    // )

                    response.add_output(SimpleImage::new(quiz.image_url(), quiz.country_code_alpha_2.clone()).build());
                    response.add_output(SimpleText::new(quiz.info_before(game.current_round)).build());
                    // response.add_output(SimpleText::new(format!("빈스 치트 - {}", quiz.answer.clone())).build());

                    debug!("->> {:<12} - bot_request - start - chat: {}, answer: {}", "HANDLER", chat_id, quiz.answer);
                }
            }

            tokio::spawn(async move {
                match gm.wait_for_answer(chat_id.clone()).await {
                    Ok(_) => {
                        // no-op
                        debug!("->> {:<12} - bot_request - wait_for_answer - solved({})", "ASYNC_HANDLER", chat_id);
                    }
                    Err(_) => {  // no one solved
                        if !gm.is_game_exists(&chat_id).await {
                            // ended already
                            return;
                        }

                        // 지금 퀴즈 답 알려주고, 다음 퀴즈로 넘어가기 (using Event API)
                        // response 말고, 별도 endpoint로 보냄 (using reqwest)
                        // Use gm.http_client
                        // assume current quiz type is same as next quiz type
                        let (current_quiz, next_quiz, next_round) = gm.get_current_quiz_and_move_next(chat_id.clone()).await.unwrap();
                        let mut async_res = Template::new();
                        match (&current_quiz, &next_quiz) {
                            (QuizType::Simple(cq), QuizType::Simple(nq)) => {
                                async_res.add_output(SimpleText::new(format!(r#"🔔 시간 초과! 정답은 "{}""#, cq.get_answer())).build());
                                if next_round > game::state::MAX_ROUNDS {
                                    async_res.add_output(SimpleText::new("✅ 다 풀었습니다 :)").build());
                                } else {
                                    async_res.add_output(SimpleText::new(nq.info_before(next_round)).build());
                                }
                            }
                            (QuizType::Flag(cq), QuizType::Flag(nq)) => {
                                async_res.add_output(SimpleText::new(format!(r#"?? 시간 초과! 정답은 "{}""#, cq.get_answer())).build());
                                if next_round > game::state::MAX_ROUNDS {
                                    async_res.add_output(SimpleText::new("✅ 다 풀었습니다 :)").build());
                                } else {
                                    async_res.add_output(SimpleImage::new(nq.image_url(), nq.country_code_alpha_2.clone()).build());
                                    async_res.add_output(SimpleText::new(nq.info_before(next_round)).build());
                                }
                            }
                            _ => {
                                // 로깅만
                                debug!("->> {:<12} - bot_request - wait_for_answer - unexpected quiz type({})", "ASYNC_HANDLER", chat_id);
                            }
                        }

                        // for test
                        debug!("->> {:<12} - bot_request - wait_for_answer - timeout({}), template: {:?}", "ASYNC_HANDLER", chat_id, async_res);

                        gm.http_client
                            .post(format!("https://bot-api.kakao.com/v2/bots/{}/talk", &config().BOT_ID))
                            .header("Authorization", format!("{} {}", &config().EVENT_API_PREFIX, &config().EVENT_API_KEY))
                            .json(&async_res)
                            .send().await.unwrap();
                    }
                }
            });
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
                    let mut result_text = format!("👏 {:.6} 정답! (누적 점수: {})", user_id, score);

                    match &current_quiz {
                        QuizType::Simple(quiz) => {
                            if let Some(comment) = quiz.comment.clone() {
                                result_text.push_str(format!("\n{}", comment).as_str());
                            }
                        }
                        QuizType::Flag(_) => {
                            // no-op
                        }
                    }

                    response.add_output(SimpleText::new(result_text).build());

                    if current_round > game::state::MAX_ROUNDS {
                        response.add_output(SimpleText::new("✅ 다 풀었습니다 :)").build());
                        gm.stop_game(chat_id.clone()).await?;
                    } else {
                        // TODO: extract
                        match &next_quiz {
                            QuizType::Simple(quiz) => {
                                response.add_output(SimpleText::new(quiz.info_before(current_round)).build());
                            }
                            QuizType::Flag(quiz) => {
                                response.add_output(SimpleImage::new(quiz.image_url(), quiz.country_code_alpha_2.clone()).build());
                                response.add_output(SimpleText::new(quiz.info_before(current_round)).build());
                                // 임시로 답도 알려준다.
                                // response.add_output(SimpleText::new(format!("빈스 치트 - {}", quiz.answer.clone())).build());
                                // outputs는 3개까지....
                            }
                        }
                    }

                    // Mark the answer as submitted.
                    gm.submit_answer(chat_id.clone()).await;
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
