use std::{thread, time::Duration};

use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};

fn get_public_ip() -> Result<String> {
    let body = reqwest::blocking::get("https://cloudflare.com/cdn-cgi/trace")?.text()?;
    let line = body.split('\n').skip(2).next();

    match line {
        // transform ip=127.0.0.1 -> 127.0.0.1
        Some(text) => Ok(text[3..].to_string()),
        None => Err(anyhow!("body is not as expected")),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct EmptyObject;

#[derive(Serialize, Deserialize, Debug)]
struct UpdateDnsRecordBody {
    comment: String,
    name: String,
    proxied: bool,
    settings: EmptyObject,
    content: String,
    ttl: u32,
    r#type: String,
}

fn update_cf_dns_record(
    auth_email: &String,
    auth_key: &String,
    zone_id: &String,
    dns_record_id: &String,
    body: UpdateDnsRecordBody,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, dns_record_id
    );

    let response = client
        .patch(url)
        .header("X-Auth-Email", auth_email)
        .header("X-Auth-Key", auth_key)
        .body(serde_json::to_string(&body)?)
        .send()?;

    info!(response:% = &response.text()?; "[update_cf_dns_record] successful");
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    zone_id: String,
    dns_record_id: String,
    cloud_flare_api_key: String,
    domain_name: String,
    interval: u64,
    proxied: bool,
    auth_email: String,
    auth_key: String,
}

/// Currently only support type A record
fn main() {
    structured_logger::Builder::with_level("info").init();

    let config = match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(err) => {
            error!(err:? = err; "[main] must have env configs is not found");
            panic!()
        }
    };

    info!("[main] started the update loop");
    let mut prev_ip = String::new();
    loop {
        let _ = match get_public_ip() {
            Ok(ip) => 'update: {
                info!(ip:% = ip; "[main] get new public ip successful");

                // skip the case when ip has not changed
                if ip == prev_ip {
                    info!(ip:% = ip; "[main] ip is the same as previous one");
                    break 'update;
                }
                prev_ip = ip.clone();

                match update_cf_dns_record(
                    &config.auth_email,
                    &config.auth_key,
                    &config.zone_id,
                    &config.dns_record_id,
                    UpdateDnsRecordBody {
                        comment: "".to_string(),
                        name: config.domain_name.clone(),
                        proxied: config.proxied,
                        settings: EmptyObject,
                        content: ip,
                        ttl: 1,
                        r#type: "A".to_string(),
                    },
                ) {
                    Ok(()) => {
                        info!("[main] update cloudflare dns record successful");
                    }
                    Err(err) => {
                        error!(err:? = err; "[main] update cloudflare dns record has failed");
                    }
                }
            }
            Err(err) => {
                error!(err:? = err; "[main] get new public ip has failed");
            }
        };
        thread::sleep(Duration::from_secs(config.interval));
    }
}
