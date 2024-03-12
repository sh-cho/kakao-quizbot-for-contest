use axum::{middleware, Router};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use crate::config::config;
use crate::game::db::{flag_quiz_db, quiz_db};
use crate::game::state::GameManager;

pub use self::error::{Error, Result};

mod web;
mod error;
mod config;
mod game;
mod conn;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    debug!("{:<12} - initializing quiz local db", "MAIN");
    quiz_db();
    flag_quiz_db();

    debug!("{:<12} - connecting to redis", "MAIN");
    let manager = RedisConnectionManager::new(config().REDIS_CONNECTION_STRING.as_str()).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();
    // {
    //     // ping the database before starting
    //     let mut conn = pool.get().await.unwrap();
    //     conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
    //     let result: String = conn.get("foo").await.unwrap();
    //     assert_eq!(result, "bar");
    // }
    // debug!("{:<12} - successfully connected to redis and pinged it", "MAIN");

    let gm = GameManager::new(pool.clone()).unwrap();
    let app = Router::new()
        .merge(web::routes_bot_request::routes(gm))
        .layer(middleware::from_fn(web::mw_auth::mw_header_checker));

    // let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on: {:?}", listener.local_addr());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
