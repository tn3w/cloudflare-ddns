mod cloudflare;
mod config;
mod ip;

use tokio::time::{sleep, Duration};

const LOGO: &str = r#"
  ____ _____   ____  ____  _   _ ____
 / ___|  ___| |  _ \|  _ \| \ | / ___|
| |   | |_    | | | | | | |  \| \___ \
| |___|  _|   | |_| | |_| | |\  |___) |
 \____|_|     |____/|____/|_| \_|____/

https://github.com/tn3w/cloudflare-ddns

Automatically update Cloudflare A records when your IP changes
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", LOGO);

    let config = config::Config::load(None)?;
    println!("Starting IP check loop with {} seconds interval", config.reload_interval);

    loop {
        match ip::get_ip().await {
            Ok(current_ip) => {
                for record_name in &config.records {
                    match cloudflare::get_dns_record(
                        config.auth_email(),
                        config.auth_key(),
                        config.zone_id(),
                        record_name,
                    )
                    .await
                    {
                        Ok(Some(record)) => {
                            if record.content != current_ip {
                                match cloudflare::update_dns_record(
                                    config.auth_email(),
                                    config.auth_key(),
                                    config.zone_id(),
                                    &record.id,
                                    &record,
                                    &current_ip,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        println!("Updated {} from {} to {}", 
                                            record.name, record.content, current_ip);
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(_) => {}
                    }
                }
            }
            Err(_) => {}
        }

        sleep(Duration::from_secs(config.reload_interval)).await;
    }
}
