use std::sync::Arc;

use tokio::{
    process::Command,
    sync::{mpsc::Receiver, Semaphore},
};
use tracing::{debug, instrument};

use crate::{Task, CONFIG};

#[instrument(skip_all)]
pub async fn executor_task(mut queue: Receiver<Task>) -> anyhow::Result<()> {
    let semaphore = Arc::new(Semaphore::new(CONFIG.executor.concurrent_limit));
    while let Some(task) = queue.recv().await {
        let semaphore = semaphore.clone();
        tokio::spawn(async move {
            let _s = semaphore.acquire().await.unwrap();
            debug!(task = ?task);
            let output = Command::new("timeout")
                .args(["10", "docker", "run", "--init"])
                .arg("--memory")
                .arg(format!("{}b", CONFIG.executor.memory_limit.as_u64()))
                .args(["--rm", "ubuntu"])
                .args(["sleep", "300"])
                .output()
                .await
                .unwrap();

            if output.status.success() {
                println!("Success!");
            } else {
                println!("Fail")
            }
            println!("stdout:{}", String::from_utf8_lossy(&output.stdout));
            println!("stderr:{}", String::from_utf8_lossy(&output.stderr));
        });
    }
    Ok(())
}
