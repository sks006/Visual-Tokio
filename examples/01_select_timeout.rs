//! 01_select_timeout.rs
//!
//! Demonstrates `tokio::select!` racing two asynchronous operations:
//! 1. A simulated database query
//! 2. A timeout deadline
//!
//! Shows cancellation-by-drop: the loser's future is dropped immediately,
//! freeing sockets, timers, and memory without leaks.

use std::time::Duration;
use tokio::time::sleep;

/// Simulates an asynchronous database query with explicit RAII drop logging.
struct DbQueryFuture {
    duration: Duration,
}

impl DbQueryFuture {
    fn new(duration: Duration) -> Self {
        Self { duration }
    }

    async fn execute(self) -> &'static str {
        println!("  [db] Query started, waiting {:?}", self.duration);
        sleep(self.duration).await;
        println!("  [db] Query completed successfully!");
        "query_result_rows"
    }
}

#[tokio::main]
async fn main() {
    println!("============================================================");
    println!(" Tokio Pattern 01: tokio::select! & Cancellation by Drop");
    println!("============================================================");

    // Scenario A: Fast query succeeds within deadline
    println!("\n▶ SCENARIO A: Fast query (100ms) vs 300ms timeout");
    let t0 = std::time::Instant::now();
    tokio::select! {
        res = DbQueryFuture::new(Duration::from_millis(100)).execute() => {
            println!("  [winner] Database query finished first: '{res}' ({:?})", t0.elapsed());
            println!("  [cleanup] Timeout future dropped immediately");
        }
        _ = sleep(Duration::from_millis(300)) => {
            println!("  [winner] Timeout elapsed!");
        }
    }

    // Scenario B: Slow query cancelled by timeout
    println!("\n▶ SCENARIO B: Slow query (800ms) vs 200ms timeout");
    let t0 = std::time::Instant::now();
    tokio::select! {
        res = DbQueryFuture::new(Duration::from_millis(800)).execute() => {
            println!("  [winner] Database query finished: '{res}'");
        }
        _ = sleep(Duration::from_millis(200)) => {
            println!("  [winner] Timeout elapsed in {:?}!", t0.elapsed());
            println!("  [cleanup] Slow DB future dropped - runtime never blocked!");
        }
    }

    println!("\n✔ Finished Pattern 01 successfully.\n");
}
