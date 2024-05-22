use anyhow::{Context, Result};
use config::get_cached_config;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};
use tokio::sync::mpsc::channel;
use tracing::{debug, instrument};

mod config;
mod executor;
mod problems;
mod users;
mod web;

#[instrument]
async fn database_init() -> Result<Pool<Sqlite>> {
    let connection_options = SqliteConnectOptions::new()
        .filename(&get_cached_config().await?.database.url)
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

    let problems = problems::ProblemsInfo::get_cached_problems_info().await?;
    for p in problems.problem.iter() {
        debug!(?p, "loaded program")
    }

    let pool = database_init().await?;

    let (tx, rx) = channel(100);

    let server_task = web::App::new(pool.clone(), tx).await?;
    let executor_task = executor::executor_task(rx, pool);

    tokio::try_join!(server_task.serve(), executor_task)?;
    Ok(())
}
