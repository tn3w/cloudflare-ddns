mod cloudflare;
mod config;
mod ip;

use env_logger::Builder;
use log::{error, info, LevelFilter};
use std::time::Instant;
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

fn setup_logger(debug: bool) {
    let mut builder = Builder::new();
    builder
        .filter_level(if debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .format_timestamp_secs()
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", LOGO);

    let config = config::Config::load(None)?;
    setup_logger(config.debug);

    info!(
        "Starting IP check loop with {} seconds interval",
        config.reload_interval
    );

    loop {
        let start = Instant::now();
        if let Some(current_ip) = ip::get_consensus_ip().await {
            info!(
                "Current IP: {} (took {:.2}s)",
                current_ip,
                start.elapsed().as_secs_f64()
            );

            for record_name in &config.records {
                let start = Instant::now();
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
                            info!(
                                "Updating {} from {} to {} (took {:.2}s)",
                                record.name,
                                record.content,
                                current_ip,
                                start.elapsed().as_secs_f64()
                            );
                            let start = Instant::now();
                            if let Err(e) = cloudflare::update_dns_record(
                                config.auth_email(),
                                config.auth_key(),
                                config.zone_id(),
                                &record.id,
                                &record,
                                &current_ip,
                            )
                            .await
                            {
                                error!(
                                    "Failed to update {}: {} (took {:.2}s)",
                                    record.name,
                                    e,
                                    start.elapsed().as_secs_f64()
                                );
                            } else {
                                info!(
                                    "Successfully updated {} (took {:.2}s)",
                                    record.name,
                                    start.elapsed().as_secs_f64()
                                );
                            }
                        } else {
                            info!(
                                "No update needed for {}, IP matches (took {:.2}s)",
                                record.name,
                                start.elapsed().as_secs_f64()
                            );
                        }
                    }
                    Ok(None) => {
                        error!("DNS record {} not found", record_name);
                    }
                    Err(e) => {
                        error!("Failed to fetch record {}: {}", record_name, e);
                    }
                }
            }
        } else {
            error!(
                "Could not determine consensus IP (took {:.2}s)",
                start.elapsed().as_secs_f64()
            );
        }

        sleep(Duration::from_secs(config.reload_interval)).await;
    }
}
