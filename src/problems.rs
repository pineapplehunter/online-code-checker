use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::{fs::read_to_string, sync::OnceCell};
use tracing::debug;

use crate::config::get_cached_config;

#[derive(Debug, Clone, Deserialize)]
pub struct Problem {
    pub group: Option<String>,
    pub name: Option<String>,
    pub id: String,
    pub dir: PathBuf,
}

impl Problem {
    pub async fn by_id(id: &str) -> Result<Self> {
        let problem = ProblemsInfo::get_cached_problems_info()
            .await?
            .problem
            .iter()
            .find(|p| p.id == id);
        if let Some(problem) = problem {
            Ok(problem.clone())
        } else {
            anyhow::bail!("problem not found")
        }
    }

    pub async fn get_index_md(&self) -> Result<String> {
        let index_md_path = get_cached_config()
            .await?
            .problems
            .dir
            .join(&self.dir)
            .join("index.md");
        Ok(read_to_string(index_md_path).await?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProblemsInfo {
    pub problem: Vec<Problem>,
}

static PROBLEMS_INFO: OnceCell<ProblemsInfo> = OnceCell::const_new();

impl ProblemsInfo {
    pub async fn get_cached_problems_info() -> Result<&'static ProblemsInfo> {
        PROBLEMS_INFO
            .get_or_try_init(|| async {
                let info_path = get_cached_config().await?.problems.dir.join("info.toml");
                debug!(?info_path);
                let problems = toml::from_str(
                    &read_to_string(info_path)
                        .await
                        .context("reading info.toml")?,
                )
                .context("parse toml")?;
                debug!(?problems);
                Ok(problems)
            })
            .await
    }
}
