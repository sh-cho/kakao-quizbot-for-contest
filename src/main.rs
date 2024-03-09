use async_trait::async_trait;
use axum::{middleware, Router};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use tracing::debug;

pub use self::error::{Error, Result};

mod web;
mod error;
mod config;
mod game;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    debug!("{:<12} - connecting to redis", "MAIN");
    let manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();
    {
        // ping the database before starting
        let mut conn = pool.get().await.unwrap();
        conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
        let result: String = conn.get("foo").await.unwrap();
        assert_eq!(result, "bar");
    }
    debug!("{:<12} - successfully connected to redis and pinged it", "MAIN");
    
    let app = Router::new()
        .merge(web::routes_bot_request::routes(pool))
        .layer(middleware::from_fn(web::mw_auth::mw_header_checker));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on: {:?}", listener.local_addr());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

pub type RedisConnectionPool = Pool<RedisConnectionManager>;

pub struct RedisConnection(PooledConnection<'static, RedisConnectionManager>);

#[async_trait]
impl<S> FromRequestParts<S> for RedisConnection
    where
        RedisConnectionPool: FromRef<S>,
        S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self> {
        let pool = RedisConnectionPool::from_ref(state);
        let conn = pool.get_owned()
            .await
            .map_err(|_| Error::RedisConnectionGetFail)?;

        Ok(Self(conn))
    }
}
