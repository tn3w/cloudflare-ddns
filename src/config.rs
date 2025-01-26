use clap::Parser;
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    TomlError(toml::de::Error),
    MissingField(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::TomlError(e) => write!(f, "TOML parsing error: {}", e),
            ConfigError::MissingField(field) => write!(f, "{} is required (use --{} or config file)", field, field),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::IoError(e) => Some(e),
            ConfigError::TomlError(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FileConfig {
    auth_email: String,
    auth_key: String,
    zone_id: String,
    reload_interval: Option<u64>,
    records: Vec<String>,
}

const LOGO: &str = r#"
  ____ _____   ____  ____  _   _ ____
 / ___|  ___| |  _ \|  _ \| \ | / ___|
| |   | |_    | | | | | | |  \| \___ \
| |___|  _|   | |_| | |_| | |\  |___) |
 \____|_|     |____/|____/|_| \_|____/
"#;

#[derive(Debug, Parser)]
#[command(author, version, about = format!("{LOGO}\nAutomatically update Cloudflare A records when your IP changes"))]
pub struct Config {
    /// Path to config file (optional if all other args are provided)
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,
    /// Cloudflare account email
    #[arg(short = 'e', long)]
    pub auth_email: Option<String>,
    /// Cloudflare API key or API token
    #[arg(short = 'k', long)]
    pub auth_key: Option<String>,
    /// Cloudflare zone ID from domain overview page
    #[arg(short = 'z', long)]
    pub zone_id: Option<String>,
    /// Update interval in seconds
    #[arg(short = 'i', long, default_value = "300")]
    pub reload_interval: u64,
    /// DNS records to update (can be specified multiple times)
    #[arg(short = 'r', long, required_unless_present = "config")]
    pub records: Vec<String>,
    /// Enable debug logging
    #[arg(short = 'd', long)]
    pub debug: bool,
}

impl Config {
    pub fn load(config_path: Option<&Path>) -> Result<Self, ConfigError> {
        let mut args = Config::parse();

        let config_path = args.config.as_deref().or(config_path);
        if let Some(path) = config_path {
            let content = fs::read_to_string(path)
                .map_err(|e| ConfigError::IoError(e))?;

            let file_config: FileConfig = toml::from_str(&content)
                .map_err(|e| ConfigError::TomlError(e))?;

            if args.auth_email.is_none() {
                args.auth_email = Some(file_config.auth_email);
            }
            if args.auth_key.is_none() {
                args.auth_key = Some(file_config.auth_key);
            }
            if args.zone_id.is_none() {
                args.zone_id = Some(file_config.zone_id);
            }
            if args.records.is_empty() {
                args.records = file_config.records;
            }
            if let Some(interval) = file_config.reload_interval {
                args.reload_interval = interval;
            }
        }

        if args.auth_email.is_none() {
            return Err(ConfigError::MissingField("auth_email"));
        }
        if args.auth_key.is_none() {
            return Err(ConfigError::MissingField("auth_key"));
        }
        if args.zone_id.is_none() {
            return Err(ConfigError::MissingField("zone_id"));
        }
        if args.records.is_empty() {
            return Err(ConfigError::MissingField("records"));
        }

        Ok(args)
    }

    pub fn auth_email(&self) -> &str {
        self.auth_email.as_ref().unwrap()
    }

    pub fn auth_key(&self) -> &str {
        self.auth_key.as_ref().unwrap()
    }

    pub fn zone_id(&self) -> &str {
        self.zone_id.as_ref().unwrap()
    }
}
