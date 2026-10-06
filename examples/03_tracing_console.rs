//! 03_tracing_console.rs
//!
//! Demonstrates structured logging with `tracing` spans, key-value fields,
//! and explains the difference between blocking worker threads vs offloading
//! with `tokio::task::spawn_blocking`.

use std::time::Duration;
use tracing::{error, info, info_span, warn, Instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// A properly behaved async function that uses async sleep without blocking.
async fn cooperative_task(id: u32) {
    let span = info_span!("cooperative_task", task_id = id);
    async {
        info!("Task started processing request");
        tokio::time::sleep(Duration::from_millis(50)).await;
        info!(status = 200, "Request handled cooperatively");
    }
    .instrument(span)
    .await;
}

/// Demonstrates how to offload heavy sync CPU/disk work using `spawn_blocking`.
async fn offloaded_blocking_task(id: u32) {
    let span = info_span!("heavy_task", task_id = id);
    async {
        info!("Offloading CPU-intensive job to threadpool");
        let result = tokio::task::spawn_blocking(move || {
            // Synchronous CPU work safely executed outside the async event loop
            std::thread::sleep(Duration::from_millis(50));
            42 * id
        })
        .await
        .expect("spawn_blocking failed");

        info!(result = result, "Worker threadpool finished computation");
    }
    .instrument(span)
    .await;
}

#[tokio::main]
async fn main() {
    // Install tracing subscriber
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    println!("============================================================");
    println!(" Tokio Pattern 03: Structured Tracing & Non-Blocking Rules");
    println!("============================================================");

    info!("Initializing Tokio tracing demo");

    // 1. Cooperative async tasks
    let h1 = tokio::spawn(cooperative_task(1));
    let h2 = tokio::spawn(cooperative_task(2));

    // 2. Offloaded blocking task
    let h3 = tokio::spawn(offloaded_blocking_task(3));

    let _ = tokio::join!(h1, h2, h3);

    warn!("Warning sample: tokio-console detects poll durations > 10ms as potential stalls!");
    error!("Rule of thumb: Never call std::thread::sleep or sync Mutex in async context; use spawn_blocking!");

    println!("\n✔ Finished Pattern 03 successfully.\n");
}
