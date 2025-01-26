mod cloudflare;
mod config;
mod ip;

use env_logger::Builder;
use log::{debug, error, info, LevelFilter};
use std::time::Instant;
use tokio::time::{sleep, Duration};

fn setup_logger(debug: bool) {
    let mut builder = Builder::new();
    builder
        .filter_level(if debug { LevelFilter::Debug } else { LevelFilter::Info })
        .format_timestamp_secs()
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                                "Updating {} from {} to {}",
                                record.name, record.content, current_ip
                            );
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
                                error!("Failed to update {}: {}", record.name, e);
                            } else {
                                info!("Successfully updated {}", record.name);
                            }
                        } else {
                            debug!("No update needed for {}, IP matches", record.name);
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
