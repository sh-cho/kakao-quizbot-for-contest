use crate::{Error, Result};
use std::env;
use std::sync::OnceLock;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|ex| {
            panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}")
        })
    })
}

#[allow(non_snake_case)]
pub struct Config {
    pub PRESHARED_AUTH_HEADER_KEY: String,
    pub PRESHARED_AUTH_HEADER_VALUE: String,
    pub REDIS_CONNECTION_STRING: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            PRESHARED_AUTH_HEADER_KEY: get_env("PRESHARED_AUTH_HEADER_KEY")?,
            PRESHARED_AUTH_HEADER_VALUE: get_env("PRESHARED_AUTH_HEADER_VALUE")?,
            REDIS_CONNECTION_STRING: get_env("REDIS_CONNECTION_STRING")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}
