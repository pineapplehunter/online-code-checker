use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use bytesize::ByteSize;
use serde::Deserialize;
use tokio::{fs::read_to_string, sync::OnceCell};
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfiguration {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecutorConfiguration {
    pub concurrent_limit: usize,
    pub memory_limit: ByteSize,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfiguration {
    pub address: String,
    pub secret_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ProblemsConfiguration {
    pub dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub database: DatabaseConfiguration,
    pub executor: ExecutorConfiguration,
    pub server: ServerConfiguration,
    pub problems: ProblemsConfiguration,
}
#[derive(Debug)]
pub struct Task {
    pub id: i64,
}

static CONFIG: OnceCell<Configuration> = OnceCell::const_new();

pub async fn get_cached_config() -> Result<&'static Configuration> {
    CONFIG
        .get_or_try_init(|| async {
            let config_filename = env::var("ONLINE_CODE_CHECKER_CONFIG")
                .unwrap_or_else(|_| "online_code_checker_config.toml".to_string());
            let config_file_constents = read_to_string(config_filename.clone())
                .await
                .with_context(|| format!("read in config file from {}", config_filename))
                .unwrap();
            let config: Configuration = toml::from_str(&config_file_constents)
                .context("parse config")
                .unwrap();
            debug!(?config);
            Ok(config)
        })
        .await
}
