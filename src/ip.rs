use reqwest::Client;
use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
pub enum IpError {
    RequestError(reqwest::Error),
    NoValidIpFound,
}

impl fmt::Display for IpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpError::RequestError(e) => write!(f, "Request error: {}", e),
            IpError::NoValidIpFound => write!(f, "No valid IP could be determined"),
        }
    }
}

impl Error for IpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IpError::RequestError(e) => Some(e),
            _ => None,
        }
    }
}

pub fn is_valid_ipv4(ip: &str) -> bool {
    Ipv4Addr::from_str(ip).is_ok()
}

async fn fetch_ip(url: &str) -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(IpError::RequestError)?
        .text()
        .await
        .map_err(IpError::RequestError)?
        .trim()
        .to_string();

    if is_valid_ipv4(&ip) {
        Ok(ip)
    } else {
        Err(IpError::NoValidIpFound)
    }
}

pub async fn get_ip() -> Result<String, IpError> {
    let services = [
        "https://api4.ipify.org",
        "https://ipv4.icanhazip.com",
        "https://ipinfo.io/ip",
        "https://ipv4.seeip.org",
        "https://ipapi.co/ip",
        "https://myip.wtf/text",
    ];

    for service in services {
        match fetch_ip(service).await {
            Ok(ip) => return Ok(ip),
            Err(_) => {}
        }
    }

    Err(IpError::NoValidIpFound)
}
