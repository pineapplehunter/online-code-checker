use std::sync::Arc;

use sqlx::{Pool, Sqlite};
use tokio::{
    process::Command,
    sync::{mpsc::Receiver, Semaphore},
};
use tracing::{debug, error, info, info_span, Instrument, Span};

use crate::{Task, CONFIG};

pub async fn executor_task(mut queue: Receiver<Task>, db: Pool<Sqlite>) -> anyhow::Result<()> {
    debug!("setup executor");

    check_docker().await;

    let semaphore = Arc::new(Semaphore::new(CONFIG.executor.concurrent_limit));
    while let Some(task) = queue.recv().await {
        let semaphore = semaphore.clone();
        tokio::spawn({
            let span = info_span!("execute task");
            async move {
                let _s = semaphore.acquire().await.unwrap();
                debug!(task = ?task);
                let output = Command::new("timeout")
                    .args(["10", "docker", "run", "--init"])
                    .arg("--memory")
                    .arg(format!("{}b", CONFIG.executor.memory_limit.as_u64()))
                    .args(["--rm", "ubuntu"])
                    .args(["echo", "hello"])
                    .output()
                    .await
                    .unwrap();

                if output.status.success() {
                    info!(
                        stdout = %String::from_utf8_lossy(&output.stdout),
                        stderr = %String::from_utf8_lossy(&output.stderr),
                        "Success!"
                    );
                } else {
                    info!("Fail")
                }
            }
            .instrument(span)
        });
    }
    Ok(())
}

async fn check_docker() {
    match Command::new("docker").arg("--version").status().await {
        Ok(_) => debug!("Found docker"),
        Err(e) => {
            error!( error = ?e,"could not find docker");
            panic!("could not find docker");
        }
    };
}
