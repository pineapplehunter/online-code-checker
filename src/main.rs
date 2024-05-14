use std::{cell::OnceCell, env, fs::read_to_string, sync::OnceLock};

use anyhow::{Context, Result};
use bytesize::ByteSize;
use once_cell::sync::Lazy;
use serde::Deserialize;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};
use tokio::sync::mpsc::channel;
use tracing::{debug, info, instrument, trace, warn};

mod executor;
mod users;
mod web;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfiguration {
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecutorConfiguration {
    concurrent_limit: usize,
    memory_limit: ByteSize,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfiguration {
    address: String,
    secret_key: String,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    database: DatabaseConfiguration,
    executor: ExecutorConfiguration,
    server: ServerConfiguration,
}
#[derive(Debug)]
pub struct Task {
    id: u32,
    filename: String,
    inputs: String,
}

pub static CONFIG: Lazy<Configuration> = Lazy::new(|| load_config());

fn load_config() -> Configuration {
    let config_filename = env::var("ONLINE_CODE_CHECKER_CONFIG")
        .unwrap_or_else(|_| "online_code_checker_config.toml".to_string());
    let config_file_constents = read_to_string(config_filename.clone())
        .with_context(|| format!("read in config file from {}", config_filename))
        .unwrap();
    let config: Configuration = toml::from_str(&config_file_constents)
        .context("parse config")
        .unwrap();
    config
}

#[instrument]
async fn database_init() -> Result<Pool<Sqlite>> {
    let connection_options = SqliteConnectOptions::new()
        .filename(&CONFIG.database.url)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(connection_options)
        .await
        .context("open databse")?;
    sqlx::migrate!()
        .run(&pool)
        .await
        .context("database migration")?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    debug!(config = ?*CONFIG);

    let pool = database_init().await?;

    let (tx, rx) = channel(100);

    let server_task = web::App::new(pool.clone(), tx).await?;
    let executor_task = executor::executor_task(rx, pool);

    tokio::try_join!(server_task.serve(), executor_task)?;
    Ok(())
}
