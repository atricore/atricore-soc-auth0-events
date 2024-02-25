use a3c_soc_auth0_events_encode_lib::EncoderType;
use clap::Parser;
use config::{Config, ConfigError, File};
use log::info;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about,
    long_about = "Atricore SOC component. Auth0 events extractor/encoder",
    arg_required_else_help = false
)]
pub struct Args {
    #[clap(short, long, required = false)]
    pub config: Option<String>,
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub address: String,
    pub api_token: String,
    pub log_config: Option<String>,
    pub encoders: Vec<EncoderType>,
}

pub fn load_config() -> Result<Settings, ConfigError> {
    // Get the file name from an argument, if not available get it from the env. var.
    // A3C_SOC_AUTH0_CONFIG_FILE
    let args = Args::parse_args();
    let config_name = args
        .config
        .or_else(|| env::var("A3C_SOC_AUTH0_EVENTS_CONFIG_FILE").ok()) // Use the environment variable if the argument is not provided
        .unwrap_or("a3c-soc-auth0-events.yaml".to_string());

    let config_path = PathBuf::from(&config_name);
    info!("Using configuration file: {}", &config_name);
    load_config_file(config_path)
}

pub fn load_config_file(config_path: PathBuf) -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(File::from(Path::new(&config_path)))
        .add_source(
            config::Environment::with_prefix("A3C_SOC_AUTH0_EVENTS")
                .try_parsing(false)
                .separator("_")
                .list_separator(" "),
        )
        .build()?
        .try_deserialize::<Settings>()
}

// Keeping tests embedded for now.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::settings::load_config_file;

    #[test]
    fn test_load_config() -> Result<(), Box<dyn std::error::Error>> {
        let f = PathBuf::from("./tests/data/settings-test-1.yml");

        load_config_file(f)
            .map(|_| Ok(()))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
    }
}
