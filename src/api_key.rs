use std::{env, fmt, fs, path::Path};

use simple_eyre::eyre::{Result, WrapErr};

pub struct ApiKey(pub String);

impl ApiKey {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("CREDENTIALS_DIRECTORY")
            .wrap_err("Environment variable CREDENTIALS_DIRECTORY not set")
            .and_then(|dir| {
                fs::read_to_string(Path::new(&dir).join("api-key"))
                    .wrap_err("Failed to read credential file")
            })
            .or_else(|_| env::var("API_KEY"))
            .wrap_err(
                "Must set either CREDENTIALS_DIRECTORY or API_KEY environment variables",
            )?;
        Ok(Self(api_key.trim().to_string()))
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (
            self.0.len() > 12,
            &self.0.get(..4),
            &self.0.get(self.0.len().saturating_sub(4)..),
        ) {
            (true, Some(start), Some(end)) => {
                let redacted = format!("{start}...(redacted)...{end}");
                write!(f, "{redacted:?}")
            }
            _ => write!(f, "\"(redacted)\""),
        }
    }
}
