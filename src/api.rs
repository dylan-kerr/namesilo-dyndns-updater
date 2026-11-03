use std::{fmt, time::Duration};

use serde::Deserialize;
use ureq::{
    Agent,
    tls::{RootCerts, TlsConfig, TlsProvider},
};

use crate::api_key::ApiKey;

const BASE_URL: &str = "https://www.namesilo.com/api";

#[derive(Deserialize, Debug)]
pub struct ListResponseRequest {
    pub ip: String,
}

#[derive(Deserialize, Debug)]
pub struct ListResponseRecord {
    pub record_id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub host: String,
    pub value: String,
    pub ttl: i32,
}

#[derive(Deserialize, Debug)]
pub struct ListResponseReply {
    pub code: i32,
    pub resource_record: Vec<ListResponseRecord>,
}

#[derive(Deserialize, Debug)]
pub struct ListResponse {
    pub request: ListResponseRequest,
    pub reply: ListResponseReply,
}

#[derive(Deserialize, Debug)]
pub struct UpdateResponseReply {
    pub code: i32,
}

#[derive(Deserialize, Debug)]
pub struct UpdateResponse {
    pub reply: UpdateResponseReply,
}

#[derive(Debug)]
pub enum Error {
    Http {
        context: &'static str,
        source: ureq::Error,
    },
    Json {
        context: &'static str,
        json: String,
        source: serde_json::Error,
    },
    Validation {
        context: &'static str,
        json: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http { context, source } => write!(f, "{context}: {source}"),
            Self::Json {
                context,
                json,
                source,
            } => write!(f, "{context}: {json}\n{source}"),
            Self::Validation { context, json } => write!(f, "{context}: {json}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Api {
    agent: Agent,
    domain: String,
    key: ApiKey,
}

impl Api {
    pub fn new(key: ApiKey, domain: String) -> Self {
        Self {
            agent: configure_agent(),
            key,
            domain,
        }
    }

    pub fn list(&self) -> Result<ListResponse, Error> {
        let list_response_json: String = retry(|| {
            self.agent
                .get(format!("{BASE_URL}/dnsListRecords"))
                .query_pairs([
                    ("version", "1"),
                    ("type", "json"),
                    ("key", &self.key.0),
                    ("domain", &self.domain),
                ])
                .call()
                .map_err(|source| Error::Http {
                    context: "Call to dnsListRecords failed",
                    source,
                })?
                .body_mut()
                .read_to_string()
                .map_err(|source| Error::Http {
                    context: "Reading dnsListRecords response failed",
                    source,
                })
        })?;
        let list_response: ListResponse =
            serde_json::from_str(&list_response_json).map_err(|source| Error::Json {
                context: "Parsing dnsListRecords response failed",
                json: list_response_json.clone(),
                source,
            })?;
        if list_response.reply.code != 300 {
            return Err(Error::Validation {
                context: "Bad response from dnsListRecords operation",
                json: list_response_json,
            });
        }
        Ok(list_response)
    }

    pub fn update(&self, record: &ListResponseRecord, ip: &str) -> Result<(), Error> {
        let update_response_json: String = retry(|| {
            self.agent
                .get(format!("{BASE_URL}/dnsUpdateRecord"))
                .query_pairs([
                    ("version", "1"),
                    ("type", "json"),
                    ("key", &self.key.0),
                    ("domain", &self.domain),
                    ("rrid", &record.record_id),
                    ("rrhost", &record.host),
                    ("rrvalue", ip),
                    ("rrttl", &record.ttl.to_string()),
                ])
                .call()
                .map_err(|source| Error::Http {
                    context: "Call to dnsUpdateRecord failed",
                    source,
                })?
                .body_mut()
                .read_to_string()
                .map_err(|source| Error::Http {
                    context: "Reading dnsUpdateRecord response failed",
                    source,
                })
        })?;
        let update_response: UpdateResponse =
            serde_json::from_str(&update_response_json).map_err(|source| Error::Json {
                context: "Parsing dnsUpdateRecord response failed",
                json: update_response_json.clone(),
                source,
            })?;
        if update_response.reply.code != 300 {
            return Err(Error::Validation {
                context: "Bad response from dnsUpdateRecord operation",
                json: update_response_json,
            });
        }
        Ok(())
    }
}

fn configure_agent() -> Agent {
    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .tls_config(
            TlsConfig::builder()
                // requires the native-tls feature
                .provider(TlsProvider::NativeTls)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into()
}

fn retry<T, F, E>(f: F) -> Result<T, E>
where
    F: Fn() -> Result<T, E>,
    E: std::error::Error,
{
    retry::retry_with_index(
        retry::delay::Exponential::from_millis(1_000).take(5),
        |attempt| {
            f().map_err(|e| {
                println!("Error on attempt {attempt}: {e:?}");
                e
            })
        },
    )
    .map_err(|e| e.error)
}
