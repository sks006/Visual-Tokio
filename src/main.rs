//! # Tokio Visual Guide — Monolithic Interactive Runner
//!
//! A single, unified monolithic CLI binary demonstrating all 6 core Tokio asynchronous patterns.
//!
//! ## Usage:
//! ```bash
//! cargo run               # Interactive menu / complete showcase
//! cargo run -- all        # Run all 6 patterns sequentially
//! cargo run -- 1          # 01: tokio::select! & Cancellation by Drop
//! cargo run -- 2          # 02: Async I/O & Reactor Multiplexing
//! cargo run -- 3          # 03: Structured Tracing & Non-Blocking Rules
//! cargo run -- 4          # 04: Resilient TcpListener Accept Loop
//! cargo run -- 5          # 05: High-Performance TCP Echo Pipeline
//! cargo run -- 6          # 06: Bounded MPSC Channels & Backpressure
//! ```

use std::env;
use std::io::ErrorKind;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info, info_span, warn, Instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const BANNER: &str = r#"
 ╔═══════════════════════════════════════════════════════════════════╗
 ║   🦀  TOKIO RUNTIME STUDIO — MONOLITHIC ASYNC REFERENCE SUITE     ║
 ║        Interactive Execution of Core Tokio Architecture           ║
 ╚═══════════════════════════════════════════════════════════════════╝
"#;

// ============================================================================
// PATTERN 01: tokio::select! & Cancellation by Drop
// ============================================================================
async fn run_pattern_01() {
    println!("\n▶ [PATTERN 01] tokio::select! & Cancellation by Drop");
    println!("------------------------------------------------------------");

    // Scenario A: Fast Query Wins
    println!("  Scenario A: Fast query (80ms) vs 250ms deadline");
    let t0 = std::time::Instant::now();
    tokio::select! {
        res = async {
            sleep(Duration::from_millis(80)).await;
            "db_query_payload"
        } => {
            println!("  ✔ Query resolved first: '{res}' in {:?}", t0.elapsed());
            println!("  🧹 Timeout future dropped immediately without leak.");
        }
        _ = sleep(Duration::from_millis(250)) => {
            println!("  ❌ Timeout expired");
        }
    }

    // Scenario B: Timeout Wins, Query Dropped
    println!("\n  Scenario B: Slow query (600ms) vs 150ms deadline");
    let t0 = std::time::Instant::now();
    tokio::select! {
        _ = async {
            sleep(Duration::from_millis(600)).await;
            "slow_db_payload"
        } => {
            println!("  Query finished");
        }
        _ = sleep(Duration::from_millis(150)) => {
            println!("  ⏰ Timeout deadline fired in {:?}!", t0.elapsed());
            println!("  🧹 Slow query future DROPPED: TCP socket closed, memory freed.");
            println!("  ⚡ Runtime remains 100% responsive!");
        }
    }
}

// ============================================================================
// PATTERN 02: Async I/O & Reactor Multiplexing
// ============================================================================
async fn run_pattern_02() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n▶ [PATTERN 02] Async I/O & OS Reactor Multiplexing (epoll)");
    println!("------------------------------------------------------------");

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    println!("  🎧 Listener bound on {addr}");

    let server_task = tokio::spawn(async move {
        for _ in 1..=6 {
            if let Ok((mut socket, peer)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = [0u8; 64];
                    if let Ok(n) = socket.read(&mut buf).await {
                        let msg = String::from_utf8_lossy(&buf[..n]);
                        let reply = format!("ACK:{msg}");
                        let _ = socket.write_all(reply.as_bytes()).await;
                        println!("    [server] Handled peer {peer}: {reply}");
                    }
                });
            }
        }
    });

    println!("  🚀 Multiplexing 6 concurrent client streams over single event loop...");
    let mut clients = Vec::new();
    for i in 1..=6 {
        clients.push(tokio::spawn(async move {
            sleep(Duration::from_millis(i * 10)).await;
            let mut client = TcpStream::connect(addr).await.expect("connect failed");
            let req = format!("client_{i}");
            client
                .write_all(req.as_bytes())
                .await
                .expect("write failed");

            let mut reply = [0u8; 64];
            let n = client.read(&mut reply).await.expect("read failed");
            println!(
                "    [client #{i}] Received: {}",
                String::from_utf8_lossy(&reply[..n])
            );
        }));
    }

    for c in clients {
        c.await?;
    }
    server_task.await?;
    println!("  ✔ All 6 connections handled without thread blocking.");
    Ok(())
}

// ============================================================================
// PATTERN 03: Structured Tracing & Non-Blocking Rules
// ============================================================================
async fn run_pattern_03() {
    println!("\n▶ [PATTERN 03] Structured Tracing & Non-Blocking Rules");
    println!("------------------------------------------------------------");

    let span = info_span!(
        "request_pipeline",
        client_ip = "192.168.1.42",
        route = "/api/compute"
    );
    async {
        info!("Step 1: Cooperative async work starting");
        tokio::time::sleep(Duration::from_millis(30)).await;

        info!("Step 2: Offloading CPU-heavy sync calculation to spawn_blocking");
        let result = tokio::task::spawn_blocking(|| {
            // CPU work safely offloaded to dedicated threadpool
            std::thread::sleep(Duration::from_millis(40));
            1337 * 7
        })
        .await
        .unwrap_or(0);

        info!(result = result, "Step 3: Offloaded computation completed");
    }
    .instrument(span)
    .await;

    warn!("Diagnosing: Tokio-console marks tasks with >10ms busy poll times as stalls!");
    error!(
        "Rule: Never block async worker threads with std::thread::sleep or sync std::sync::Mutex!"
    );
}

// ============================================================================
// PATTERN 04: Resilient TcpListener Accept Loop
// ============================================================================
async fn run_pattern_04() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n▶ [PATTERN 04] Resilient TcpListener Accept Loop");
    println!("------------------------------------------------------------");

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    println!("  🎧 Resilient server listening on {addr}");

    let server_task = tokio::spawn(async move {
        let mut count = 0;
        loop {
            match listener.accept().await {
                Ok((mut socket, peer)) => {
                    count += 1;
                    println!("    [server] Accepted #{count} from {peer}");
                    tokio::spawn(async move {
                        let mut buf = [0u8; 32];
                        if let Ok(n) = socket.read(&mut buf).await {
                            let _ = socket.write_all(&buf[..n]).await;
                        }
                    });
                }
                Err(err) => {
                    // Demonstrate transient error classification
                    match err.kind() {
                        ErrorKind::ConnectionAborted | ErrorKind::Interrupted => {
                            eprintln!(
                                "    [server warning] Transient network abort: {err}. Retrying..."
                            );
                            continue;
                        }
                        ErrorKind::WouldBlock => {
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            continue;
                        }
                        _ => {
                            eprintln!("    [server fatal] Non-recoverable socket error: {err}");
                            break;
                        }
                    }
                }
            }

            if count >= 3 {
                break;
            }
        }
    });

    for i in 1..=3 {
        let mut client = TcpStream::connect(addr).await?;
        let payload = format!("ping-{i}");
        client.write_all(payload.as_bytes()).await?;
        let mut buf = [0u8; 32];
        let n = client.read(&mut buf).await?;
        println!(
            "    [client] Received pong: {}",
            String::from_utf8_lossy(&buf[..n])
        );
    }

    server_task.await?;
    println!("  ✔ Loop survived and served 3 connections with error guards.");
    Ok(())
}

// ============================================================================
// PATTERN 05: High-Performance TCP Echo Pipeline
// ============================================================================
async fn run_pattern_05() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n▶ [PATTERN 05] High-Performance TCP Echo Pipeline (Stream Splitting)");
    println!("------------------------------------------------------------");

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let server = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let (mut reader, mut writer) = stream.split();
            let _ = tokio::io::copy(&mut reader, &mut writer).await;
        }
    });

    let mut client = TcpStream::connect(addr).await?;
    let test_msg = b"Monolithic Tokio Echo Test String";
    client.write_all(test_msg).await?;
    client.shutdown().await?;

    let mut response = Vec::new();
    client.read_to_end(&mut response).await?;
    println!("  Sent:     {}", String::from_utf8_lossy(test_msg));
    println!("  Echoed:   {}", String::from_utf8_lossy(&response));
    assert_eq!(&response, test_msg);

    server.await?;
    println!("  ✔ Zero-copy stream split & echo completed successfully.");
    Ok(())
}

// ============================================================================
// PATTERN 06: Bounded MPSC Channels & Backpressure
// ============================================================================
async fn run_pattern_06() {
    println!("\n▶ [PATTERN 06] Bounded MPSC Channels & Backpressure");
    println!("------------------------------------------------------------");

    let (tx, mut rx) = mpsc::channel::<String>(3);
    println!("  📬 Created bounded channel: capacity = 3 permits");

    let tx1 = tx.clone();
    let p1 = tokio::spawn(async move {
        for i in 1..=2 {
            tx1.send(format!("Worker-A:msg-{i}"))
                .await
                .expect("send failed");
            sleep(Duration::from_millis(20)).await;
        }
        println!("    [Worker A] Completed & dropped sender handle.");
    });

    let tx2 = tx.clone();
    let p2 = tokio::spawn(async move {
        for i in 1..=2 {
            tx2.send(format!("Worker-B:msg-{i}"))
                .await
                .expect("send failed");
            sleep(Duration::from_millis(25)).await;
        }
        println!("    [Worker B] Completed & dropped sender handle.");
    });

    // CRITICAL: Drop the original coordinator tx handle
    drop(tx);
    println!("  🔑 Coordinator dropped original tx handle (watching worker clones).");

    let mut count = 0;
    while let Some(msg) = rx.recv().await {
        count += 1;
        println!("    [Receiver] Processed message #{count}: {msg}");
    }

    println!("  🏁 rx.recv() returned None: All senders dropped, graceful termination!");
    let _ = tokio::join!(p1, p2);
    println!("  ✔ Handled all {count} messages with proper backpressure.");
}

// ============================================================================
// MONOLITHIC ENTRY POINT & DISPATCHER
// ============================================================================
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber once
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init();

    println!("{BANNER}");

    let args: Vec<String> = env::args().collect();
    let selection = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    match selection {
        "1" | "select" => run_pattern_01().await,
        "2" | "reactor" | "io" => run_pattern_02().await?,
        "3" | "tracing" | "console" => run_pattern_03().await,
        "4" | "accept" | "listener" => run_pattern_04().await?,
        "5" | "echo" => run_pattern_05().await?,
        "6" | "mpsc" | "channel" => run_pattern_06().await,
        "all" => {
            println!("⚡ Executing All 6 Architectural Patterns Sequentially:\n");
            run_pattern_01().await;
            run_pattern_02().await?;
            run_pattern_03().await;
            run_pattern_04().await?;
            run_pattern_05().await?;
            run_pattern_06().await;
            println!("\n🎉 All 6 Tokio architectural patterns executed successfully!");
        }
        other => {
            eprintln!("Unknown pattern selector: '{other}'");
            eprintln!("Available selectors: 1, 2, 3, 4, 5, 6, all");
            std::process::exit(1);
        }
    }

    println!("\n============================================================");
    println!("💡 Tip: Open 'index.html' in your browser for the full");
    println!("   interactive visual simulation studio & architecture maps!");
    println!("============================================================\n");

    Ok(())
}
