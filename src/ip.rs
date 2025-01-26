use futures::future::{join_all, BoxFuture};
use log::debug;
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
}

impl fmt::Display for IpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpError::RequestError(e) => write!(f, "Request error: {}", e),
        }
    }
}

impl Error for IpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            IpError::RequestError(e) => Some(e),
        }
    }
}

pub fn is_valid_ipv4(ip: &str) -> bool {
    Ipv4Addr::from_str(ip).is_ok()
}

pub async fn get_ip_ipify() -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get("https://api4.ipify.org")
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

pub async fn get_ip_ipapi() -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get("https://ipapi.co/ip")
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

pub async fn get_ip_seeip() -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get("https://ipv4.seeip.org")
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

pub async fn get_ip_ipinfo() -> Result<String, IpError> {
    let client = Client::new();
    let ip = client
        .get("https://ipinfo.io/ip")
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

pub async fn get_consensus_ip() -> Option<String> {
    let results = Arc::new(Mutex::new(Vec::new()));

    let sources: [(_, BoxFuture<'static, Result<String, IpError>>); 6] = [
        ("ipapi.co", Box::pin(get_ip_ipapi())),
        ("ipify", Box::pin(get_ip_ipify())),
        ("icanhazip", Box::pin(get_ip_icanhazip())),
        ("seeip", Box::pin(get_ip_seeip())),
        ("ipinfo", Box::pin(get_ip_ipinfo())),
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
                        debug!("Error: Invalid IPv4 from {}: {}", source_name, ip);
                    }
                    Err(e) => {
                        debug!("Error: Failed to get IP from {}: {}", source_name, e);
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

    None
}
