use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::debug;

use crate::{Error, Result};
use crate::config::config;

pub async fn mw_header_checker(
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_header_checker", "MIDDLEWARE");

    let auth_header = req.headers().get(&config().PRESHARED_AUTH_HEADER_KEY);
    if auth_header.is_none() || auth_header.unwrap() != &config().PRESHARED_AUTH_HEADER_VALUE {
        return Err(Error::AuthFail);
    }

    Ok(next.run(req).await)
}
