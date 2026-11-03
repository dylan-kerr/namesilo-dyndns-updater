use std::env;

use crate::api_key::ApiKey;

#[derive(Debug)]
pub struct Config {
    pub domain: String,
    pub subdomain: String,
    pub api_key: ApiKey,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            domain: get_env_var("DOMAIN")?,
            subdomain: get_env_var("SUBDOMAIN")?,
            api_key: ApiKey::from_env()?,
        })
    }
}

fn get_env_var(key: &str) -> Result<String, String> {
    env::var(key)
        .map(|v| v.trim().to_string())
        .map_err(|_| format!("Must set environment variable {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_env {
        use super::*;
        use crate::test_helpers::run_env_var_test;

        #[test]
        fn trims_whitespace() {
            let config = run_env_var_test(|| {
                unsafe {
                    env::set_var("DOMAIN", " domain ");
                    env::set_var("SUBDOMAIN", " subdomain ");
                    env::remove_var("CREDENTIALS_DIRECTORY");
                    env::set_var("API_KEY", " api key ");
                }

                Config::from_env().unwrap()
            });

            assert_eq!(config.domain, "domain");
            assert_eq!(config.subdomain, "subdomain");
            assert_eq!(config.api_key.0, "api key");
        }

        #[test]
        fn returns_err_if_domain_not_specified() {
            let result = run_env_var_test(|| {
                unsafe {
                    env::remove_var("DOMAIN");
                    env::set_var("SUBDOMAIN", " subdomain ");
                    env::remove_var("CREDENTIALS_DIRECTORY");
                    env::set_var("API_KEY", " api key ");
                }

                Config::from_env()
            });

            assert!(result.is_err());
        }

        #[test]
        fn return_err_if_subdomain_not_specified() {
            let result = run_env_var_test(|| {
                unsafe {
                    env::set_var("DOMAIN", " domain ");
                    env::remove_var("SUBDOMAIN");
                    env::remove_var("CREDENTIALS_DIRECTORY");
                    env::set_var("API_KEY", " api key ");
                }

                Config::from_env()
            });

            assert!(result.is_err());
        }

        #[test]
        fn returns_err_if_api_key_not_specified() {
            let result = run_env_var_test(|| {
                unsafe {
                    env::set_var("DOMAIN", " domain ");
                    env::set_var("SUBDOMAIN", " subdomain ");
                    env::remove_var("CREDENTIALS_DIRECTORY");
                    env::remove_var("API_KEY");
                }

                Config::from_env()
            });

            assert!(result.is_err());
        }
    }
}
