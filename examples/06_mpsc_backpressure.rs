//! 06_mpsc_backpressure.rs
//!
//! Demonstrates Tokio's bounded multi-producer, single-consumer channel (`mpsc`):
//! - Buffer capacity and permit acquisition
//! - Backpressure when the channel fills up
//! - Sender cloning (`tx.clone()`) and reference counting
//! - The critical rule: `rx.recv()` only returns `None` after ALL `Sender` handles are dropped!

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("============================================================");
    println!(" Tokio Pattern 06: Bounded MPSC Channels & Backpressure");
    println!("============================================================");

    // Create a bounded channel with capacity 4
    let (tx, mut rx) = mpsc::channel::<String>(4);
    println!("▶ Created mpsc::channel with capacity 4");

    // Spawn Producer 1
    let tx1 = tx.clone();
    let p1 = tokio::spawn(async move {
        for i in 1..=3 {
            println!("  [producer 1] Sending task 1.{i}");
            tx1.send(format!("P1:item-{i}")).await.expect("send failed");
            sleep(Duration::from_millis(20)).await;
        }
        println!("  [producer 1] Done, dropping tx1");
    });

    // Spawn Producer 2
    let tx2 = tx.clone();
    let p2 = tokio::spawn(async move {
        for i in 1..=3 {
            println!("  [producer 2] Sending task 2.{i}");
            tx2.send(format!("P2:item-{i}")).await.expect("send failed");
            sleep(Duration::from_millis(30)).await;
        }
        println!("  [producer 2] Done, dropping tx2");
    });

    // CRITICAL: Drop the original `tx` handle in the coordinator,
    // otherwise the receiver loop will wait forever for more messages!
    drop(tx);
    println!("▶ Dropped original tx handle (now waiting on tx1 and tx2)");

    // Receiver drains the channel
    let mut received_count = 0;
    while let Some(msg) = rx.recv().await {
        received_count += 1;
        println!("  [consumer] Processed message #{received_count}: {msg}");
    }

    println!("▶ rx.recv() returned None: All sender handles dropped, channel cleanly closed!");

    let _ = tokio::join!(p1, p2);
    println!(
        "\n✔ Finished Pattern 06: Handled {received_count} messages with graceful shutdown.\n"
    );
}
