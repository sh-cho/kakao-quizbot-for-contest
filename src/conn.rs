use bb8::Pool;
use bb8_redis::RedisConnectionManager;

pub type RedisConnectionPool = Pool<RedisConnectionManager>;

// pub struct RedisConnection(pub PooledConnection<'static, RedisConnectionManager>);

// #[async_trait]
// impl<S> FromRequestParts<S> for RedisConnection
//     where
//         RedisConnectionPool: FromRef<S>,
//         S: Send + Sync,
// {
//     type Rejection = Error;
// 
//     async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self> {
//         let pool = RedisConnectionPool::from_ref(state);
//         let conn = pool.get_owned()
//             .await
//             .map_err(|_| Error::RedisConnectionGetFail)?;
// 
//         Ok(Self(conn))
//     }
// }

