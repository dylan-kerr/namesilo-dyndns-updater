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
            .wrap_err("Must set either CREDENTIALS_DIRECTORY or API_KEY environment variables")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    mod from_env {
        use std::sync::Mutex;

        use super::*;

        static SERIAL_GUARD: Mutex<()> = Mutex::new(());

        #[test]
        fn returns_key_from_credential_file_if_available() {
            let temp_dir = tempfile::tempdir().unwrap();
            fs::write(
                temp_dir.path().to_owned().join("api-key"),
                " from credential ",
            )
            .unwrap();

            let guard = SERIAL_GUARD.lock().unwrap();
            unsafe {
                env::set_var("CREDENTIALS_DIRECTORY", temp_dir.path());
                env::set_var("API_KEY", " from env var ");
            }

            let api_key = ApiKey::from_env().unwrap();

            drop(guard);

            assert_eq!(api_key.0, "from credential");
        }

        #[test]
        fn returns_key_from_env_var_if_credential_file_missing() {
            let temp_dir = tempfile::tempdir().unwrap();

            let guard = SERIAL_GUARD.lock().unwrap();
            unsafe {
                env::set_var("CREDENTIALS_DIRECTORY", temp_dir.path());
                env::set_var("API_KEY", " from env var ");
            }

            let api_key = ApiKey::from_env().unwrap();

            drop(guard);

            assert_eq!(api_key.0, "from env var");
        }

        #[test]
        fn returns_key_from_env_var_if_credential_dir_not_specified() {
            let guard = SERIAL_GUARD.lock().unwrap();
            unsafe {
                env::remove_var("CREDENTIALS_DIRECTORY");
                env::set_var("API_KEY", " from env var ");
            }

            let api_key = ApiKey::from_env().unwrap();

            drop(guard);

            assert_eq!(api_key.0, "from env var");
        }

        #[test]
        fn returns_err_if_neither_env_var_specified() {
            let guard = SERIAL_GUARD.lock().unwrap();
            unsafe {
                env::remove_var("CREDENTIALS_DIRECTORY");
                env::remove_var("API_KEY");
            }

            let result = ApiKey::from_env();

            drop(guard);

            assert!(result.is_err());
        }
    }

    mod fmt {
        use super::*;

        #[test]
        fn shows_the_start_and_end_of_a_long_key() {
            let api_key = ApiKey("1234 long enough key 1234".to_string());

            let result = format!("{api_key:?}");

            assert_eq!(result, "\"1234...(redacted)...1234\"");
        }

        #[test]
        fn shows_none_of_a_short_key() {
            let api_key = ApiKey("too short 12".to_string());

            let result = format!("{api_key:?}");

            assert_eq!(result, "\"(redacted)\"");
        }

        #[test]
        fn safely_handles_multibyte_chars_at_start() {
            // 🌏 is 4 bytes, so taking the first 4 bytes would be invalid.
            let api_key = ApiKey("1🌏 otherwise long enough 1234".to_string());

            let result = format!("{api_key:?}");

            assert_eq!(result, "\"(redacted)\"");
        }

        #[test]
        fn safely_handles_multibyte_chars_at_end() {
            // 🌏 is 4 bytes, so taking the last 4 bytes would be invalid.
            let api_key = ApiKey("1234 otherwise long enough 🌏4".to_string());

            let result = format!("{api_key:?}");

            assert_eq!(result, "\"(redacted)\"");
        }

        #[test]
        fn safely_handles_empty_key() {
            let api_key = ApiKey(String::new());

            let result = format!("{api_key:?}");

            assert_eq!(result, "\"(redacted)\"");
        }
    }
}
