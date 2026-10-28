use std::time::Duration;

use serde::Deserialize;
use simple_eyre::eyre::{Result, WrapErr, eyre};
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

    pub fn list(&self) -> Result<ListResponse> {
        let list_response_json: String = self
            .agent
            .get(format!("{BASE_URL}/dnsListRecords"))
            .query_pairs(vec![
                ("version", "1"),
                ("type", "json"),
                ("key", &self.key.0),
                ("domain", &self.domain),
            ])
            .call()
            .wrap_err("Call to dnsListRecords failed")?
            .body_mut()
            .read_to_string()
            .wrap_err("Reading dnsListRecords response failed")?;
        let list_response: ListResponse =
            serde_json::from_str(&list_response_json).wrap_err_with(|| {
                format!("Parsing dnsListRecords response failed: {list_response_json}")
            })?;
        if list_response.reply.code != 300 {
            return Err(eyre!(
                "Bad response from dnsListRecords operation: {list_response_json}"
            ));
        }
        Ok(list_response)
    }

    pub fn update(&self, record: &ListResponseRecord, ip: &str) -> Result<()> {
        let update_response_json: String = self
            .agent
            .get(format!("{BASE_URL}/dnsUpdateRecord"))
            .query_pairs(vec![
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
            .wrap_err("Call to dnsUpdateRecord failed")?
            .body_mut()
            .read_to_string()
            .wrap_err("Reading dnsUpdateRecord response failed")?;
        let update_response: UpdateResponse = serde_json::from_str(&update_response_json)
            .wrap_err_with(|| {
                format!("Parsing dnsUpdateRecord response failed: {update_response_json}")
            })?;
        if update_response.reply.code != 300 {
            return Err(eyre!(
                "Bad response from dnsUpdateRecord operation: {update_response_json}"
            ));
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
