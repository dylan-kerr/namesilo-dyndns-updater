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

#[cfg(test)]
mod tests {
    use super::*;

    mod from_env {
        use std::sync::Mutex;

        use super::*;

        static SERIAL_GUARD: Mutex<()> = Mutex::new(());

        #[test]
        fn trims_whitespace() {
            let guard = SERIAL_GUARD.lock().unwrap();
            unsafe {
                env::set_var("DOMAIN", " domain ");
                env::set_var("SUBDOMAIN", " subdomain ");
                env::remove_var("CREDENTIALS_DIRECTORY");
                env::set_var("API_KEY", " api key ");
            }

            let config = Config::from_env().unwrap();

            drop(guard);

            assert_eq!(config.domain, "domain");
            assert_eq!(config.subdomain, "subdomain");
            assert_eq!(config.api_key.0, "api key");
        }

        #[test]
        fn returns_err_if_domain_not_specified() {
            let guard = SERIAL_GUARD.lock().unwrap();
            unsafe {
                env::remove_var("DOMAIN");
                env::set_var("SUBDOMAIN", " subdomain ");
                env::remove_var("CREDENTIALS_DIRECTORY");
                env::set_var("API_KEY", " api key ");
            }

            let result = Config::from_env();

            drop(guard);

            assert!(result.is_err());
        }

        #[test]
        fn return_err_if_subdomain_not_specified() {
            let guard = SERIAL_GUARD.lock().unwrap();
            unsafe {
                env::set_var("DOMAIN", " domain ");
                env::remove_var("SUBDOMAIN");
                env::remove_var("CREDENTIALS_DIRECTORY");
                env::set_var("API_KEY", " api key ");
            }

            let result = Config::from_env();

            drop(guard);

            assert!(result.is_err());
        }

        #[test]
        fn returns_err_if_api_key_not_specified() {
            let guard = SERIAL_GUARD.lock().unwrap();
            unsafe {
                env::set_var("DOMAIN", " domain ");
                env::set_var("SUBDOMAIN", " subdomain ");
                env::remove_var("CREDENTIALS_DIRECTORY");
                env::remove_var("API_KEY");
            }

            let result = Config::from_env();

            drop(guard);

            assert!(result.is_err());
        }
    }
}
