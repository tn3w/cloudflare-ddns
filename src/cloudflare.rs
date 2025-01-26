use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CloudflareError {
    ApiError(String, i32),
    RequestError(reqwest::Error),
    UnknownError,
}

impl fmt::Display for CloudflareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CloudflareError::ApiError(msg, code) => {
                write!(f, "Cloudflare API error: {} (code: {})", msg, code)
            }
            CloudflareError::RequestError(e) => write!(f, "Request error: {}", e),
            CloudflareError::UnknownError => write!(f, "Unknown error"),
        }
    }
}

impl Error for CloudflareError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CloudflareError::RequestError(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub content: String,
    pub proxied: bool,
    pub ttl: u32,
}

#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareResponseError>,
    result: Option<Vec<T>>,
}

#[derive(Debug, Deserialize)]
struct CloudflareResponseError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct UpdateDnsRecord {
    content: String,
    name: String,
    proxied: bool,
    ttl: u32,
    #[serde(rename = "type")]
    record_type: String,
}

#[derive(Debug, Deserialize)]
struct UpdateResponse {
    success: bool,
    errors: Vec<CloudflareResponseError>,
}

fn is_global_api_key(key: &str) -> bool {
    key.len() == 37 && key.chars().all(|c| c.is_ascii_hexdigit())
}

pub async fn get_dns_record(
    auth_email: &str,
    auth_key: &str,
    zone_id: &str,
    record_name: &str,
) -> Result<Option<DnsRecord>, CloudflareError> {
    let client = Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone_id
    );

    debug!("Fetching DNS record for {}", record_name);

    let mut request = client
        .get(&url)
        .query(&[("type", "A"), ("name", record_name)])
        .header("X-Auth-Email", auth_email);

    request = if is_global_api_key(auth_key) {
        request.header("X-Auth-Key", auth_key)
    } else {
        request.header("Authorization", format!("Bearer {}", auth_key))
    };

    let response: CloudflareResponse<DnsRecord> = request
        .send()
        .await
        .map_err(CloudflareError::RequestError)?
        .json()
        .await
        .map_err(CloudflareError::RequestError)?;

    if !response.success {
        let error = response
            .errors
            .first()
            .map(|e| CloudflareError::ApiError(e.message.clone(), e.code))
            .unwrap_or(CloudflareError::UnknownError);
        return Err(error);
    }

    Ok(response.result.and_then(|mut records| records.pop()))
}

pub async fn update_dns_record(
    auth_email: &str,
    auth_key: &str,
    zone_id: &str,
    record_id: &str,
    record: &DnsRecord,
    new_ip: &str,
) -> Result<(), CloudflareError> {
    let client = Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record_id
    );

    debug!("Updating DNS record {} with IP {}", record.name, new_ip);

    let update_data = UpdateDnsRecord {
        content: new_ip.to_string(),
        name: record.name.clone(),
        proxied: record.proxied,
        ttl: record.ttl,
        record_type: "A".to_string(),
    };

    let mut request = client
        .put(&url)
        .header("X-Auth-Email", auth_email)
        .json(&update_data);

    request = if is_global_api_key(auth_key) {
        request.header("X-Auth-Key", auth_key)
    } else {
        request.header("Authorization", format!("Bearer {}", auth_key))
    };

    let response: UpdateResponse = request
        .send()
        .await
        .map_err(CloudflareError::RequestError)?
        .json()
        .await
        .map_err(CloudflareError::RequestError)?;

    if !response.success {
        let error = response
            .errors
            .first()
            .map(|e| CloudflareError::ApiError(e.message.clone(), e.code))
            .unwrap_or(CloudflareError::UnknownError);
        return Err(error);
    }

    Ok(())
}
