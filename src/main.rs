use simple_eyre::eyre::{OptionExt, Result};

use crate::{
    api::{Api, ListResponse, ListResponseRecord},
    config::Config,
};

mod api;
mod api_key;
mod config;

fn find_matching_record<'a>(
    list_response: &'a ListResponse,
    subdomain: &str,
) -> Result<&'a ListResponseRecord> {
    list_response
        .reply
        .resource_record
        .iter()
        .find(|&r| r.type_ == "A" && r.host == subdomain)
        .ok_or_eyre("No matching record in API response")
}

fn main() -> Result<()> {
    simple_eyre::install()?;

    let config = Config::from_env()?;
    let api = Api::new(config.api_key, config.domain);

    let list_response = api.list()?;

    let matching_record = find_matching_record(&list_response, &config.subdomain)?;

    if list_response.request.ip == matching_record.value {
        println!("IP address unchanged, exiting");
        return Ok(());
    }
    println!(
        "Namesilo has {}, current IP is {}. Updating Namesilo...",
        matching_record.value, list_response.request.ip
    );

    api.update(matching_record, &list_response.request.ip)?;

    println!("Successfully updated Namesilo");
    Ok(())
}
