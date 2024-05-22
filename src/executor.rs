use std::{process::Stdio, sync::Arc};

use sqlx::{Pool, Sqlite};
use tokio::{
    io::AsyncWriteExt,
    process::Command,
    sync::{mpsc::Receiver, Semaphore},
};
use tracing::{debug, error, info, info_span, Instrument};

use crate::config::{get_cached_config, Task};

const NIX_FILE: &str = include_str!("embedded-default.nix");

pub async fn executor_task(mut queue: Receiver<Task>, db: Pool<Sqlite>) -> anyhow::Result<()> {
    debug!("setup executor");

    check_docker().await;

    let semaphore = Arc::new(Semaphore::new(
        get_cached_config().await?.executor.concurrent_limit,
    ));
    while queue.recv().await.is_some() {
        let tasks = sqlx::query!("select id from solutions where status = \"Pending\"")
            .fetch_all(&db)
            .await?;
        if tasks.is_empty() {
            break;
        }
        for task in tasks {
            let semaphore = semaphore.clone();
            tokio::spawn({
                let span = info_span!("execute task");
                let db = db.clone();
                async move {
                    let _s = semaphore.acquire().await.unwrap();
                    debug!(task = ?task);
                    let tmp = tempdir::TempDir::new("OCC").unwrap();

                    let record = sqlx::query!(
                        "select problem_id,content from solutions where id = ?",
                        task.id
                    )
                    .fetch_one(&db)
                    .await
                    .unwrap();
                    tokio::fs::File::create(tmp.path().join("default.nix"))
                        .await
                        .unwrap()
                        .write_all(NIX_FILE.as_bytes())
                        .await
                        .unwrap();
                    tokio::fs::File::create(tmp.path().join("main.c"))
                        .await
                        .unwrap()
                        .write_all(record.content.unwrap().as_bytes())
                        .await
                        .unwrap();
                    let mut readdir = tokio::fs::read_dir(
                        get_cached_config()
                            .await
                            .unwrap()
                            .problems
                            .dir
                            .join(record.problem_id),
                    )
                    .await
                    .unwrap();
                    while let Ok(Some(entry)) = readdir.next_entry().await {
                        tokio::fs::copy(entry.path(), tmp.path().join(entry.file_name()))
                            .await
                            .unwrap();
                    }

                    let output = Command::new("timeout")
                        .args(["300", "docker", "run", "--init"])
                        .args(["-v", &format!("{}:/check", tmp.path().to_str().unwrap())])
                        .args(["--rm", "ghcr.io/nixos/nix"])
                        .args(["nix-build", "check"])
                        .output()
                        .await
                        .unwrap();

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if output.status.success() {
                        info!(
                            stdout = %stdout,
                            stderr = %stderr,
                            "Success!"
                        );
                        sqlx::query!("update solutions set status = ?, stdout = ?, stderr = ? where id = ?", "AC", stdout, stderr, task.id)
                            .execute(&db)
                            .await
                            .unwrap();
                    } else {
                        info!(
                            stdout = %stdout,
                            stderr = %stderr,
                            "Fail"
                        );
                        sqlx::query!("update solutions set status = ?, stdout = ?, stderr = ? where id = ?", "WA", stdout, stderr, task.id)
                            .execute(&db)
                            .await
                            .unwrap();
                    }
                }
                .instrument(span)
            });
        }
    }
    Ok(())
}

async fn check_docker() {
    match Command::new("docker")
        .arg("--version")
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
    {
        Ok(_) => debug!("Found docker"),
        Err(e) => {
            error!( error = ?e,"could not find docker");
            panic!("could not find docker");
        }
    };
}
