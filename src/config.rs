use std::env;

use simple_eyre::eyre::{Result, WrapErr};

use crate::api_key::ApiKey;

#[derive(Debug)]
pub struct Config {
    pub domain: String,
    pub subdomain: String,
    pub api_key: ApiKey,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            domain: get_env_var("DOMAIN")?,
            subdomain: get_env_var("SUBDOMAIN")?,
            api_key: ApiKey::from_env()?,
        })
    }
}

fn get_env_var(key: &str) -> Result<String> {
    env::var(key)
        .map(|v| v.trim().to_string())
        .wrap_err(format!("Must set environment variable {key}"))
}
