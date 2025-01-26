use futures::future::{join_all, BoxFuture};
use log::{debug, error};
use reqwest::Client;
use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum IpError {
    RequestError(reqwest::Error),
    ParseError(&'static str),
}

impl fmt::Display for IpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpError::RequestError(e) => write!(f, "Request error: {}", e),
            IpError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl Error for IpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IpError::RequestError(e) => Some(e),
            IpError::ParseError(_) => None,
        }
    }
}

pub fn is_valid_ipv4(ip: &str) -> bool {
    Ipv4Addr::from_str(ip).is_ok()
}

pub async fn get_ip_cloudflare() -> Result<String, IpError> {
    let client = Client::new();
    let response = client
        .get("https://cloudflare.com/cdn-cgi/trace")
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .map_err(IpError::RequestError)?
        .text()
        .await
        .map_err(IpError::RequestError)?;

    response
        .lines()
        .find(|line| line.starts_with("ip="))
        .map(|line| line[3..].to_string())
        .ok_or_else(|| IpError::ParseError("Could not find IP in Cloudflare response"))
}

pub async fn get_ip_ipify() -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get("https://api.ipify.org")
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .map_err(IpError::RequestError)?
        .text()
        .await
        .map_err(IpError::RequestError)?
        .trim()
        .to_string();
    Ok(ip)
}

pub async fn get_ip_icanhazip() -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get("https://ipv4.icanhazip.com")
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .map_err(IpError::RequestError)?
        .text()
        .await
        .map_err(IpError::RequestError)?
        .trim()
        .to_string();
    Ok(ip)
}

pub async fn get_ip_ipme() -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get("https://ip.me")
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .map_err(IpError::RequestError)?
        .text()
        .await
        .map_err(IpError::RequestError)?
        .trim()
        .to_string();
    Ok(ip)
}

pub async fn get_ip_ifconfig() -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get("https://ifconfig.me/ip")
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .map_err(IpError::RequestError)?
        .text()
        .await
        .map_err(IpError::RequestError)?
        .trim()
        .to_string();
    Ok(ip)
}

pub async fn get_ip_myipwtf() -> Result<String, IpError> {
    let client = Client::new();
    let response = client
        .get("https://myip.wtf/text")
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .map_err(IpError::RequestError)?
        .text()
        .await
        .map_err(IpError::RequestError)?;
    Ok(response.trim().to_string())
}

pub async fn get_consensus_ip() -> Option<String> {
    let results = Arc::new(Mutex::new(Vec::new()));

    let sources: [(_, BoxFuture<'static, Result<String, IpError>>); 6] = [
        ("Cloudflare", Box::pin(get_ip_cloudflare())),
        ("IPify", Box::pin(get_ip_ipify())),
        ("ICanHazIP", Box::pin(get_ip_icanhazip())),
        ("IP.me", Box::pin(get_ip_ipme())),
        ("ifconfig.me", Box::pin(get_ip_ifconfig())),
        ("myip.wtf", Box::pin(get_ip_myipwtf())),
    ];

    let futures: Vec<_> = sources
        .into_iter()
        .map(|(source_name, future)| {
            let results = Arc::clone(&results);
            tokio::spawn(async move {
                match future.await {
                    Ok(ip) if is_valid_ipv4(&ip) => {
                        debug!("Got IP from {}: {}", source_name, ip);
                        let mut results = results.lock().await;
                        results.push(ip.clone());

                        for ip in results.iter() {
                            let count = results.iter().filter(|&x| x == ip).count();
                            if count >= 2 {
                                debug!("Found consensus IP early: {}", ip);
                                return Some(ip.clone());
                            }
                        }
                    }
                    Ok(ip) => {
                        error!("Invalid IPv4 from {}: {}", source_name, ip);
                    }
                    Err(e) => {
                        error!("Failed to get IP from {}: {}", source_name, e);
                    }
                }
                None
            })
        })
        .collect();

    for result in join_all(futures).await {
        if let Ok(Some(ip)) = result {
            return Some(ip);
        }
    }

    let final_results = results.lock().await;
    for ip in final_results.iter() {
        let count = final_results.iter().filter(|&x| x == ip).count();
        if count >= 2 {
            debug!("Found consensus IP in final check: {}", ip);
            return Some(ip.clone());
        }
    }

    error!("Could not get consensus IP (at least 2 matching sources required)");
    None
}
