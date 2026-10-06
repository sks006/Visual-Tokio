//! 04_tcp_accept_resilient.rs
//!
//! Demonstrates why a naive `while let Ok((socket, addr)) = listener.accept().await`
//! can prematurely exit the entire server loop on transient OS errors (e.g. ECONNABORTED,
//! EMFILE), and how to write a production-grade resilient accept loop.

use std::io::ErrorKind;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

/// A resilient connection handler that does not crash on transient network errors.
async fn run_resilient_server(listener: TcpListener) {
    let mut accepted_count = 0;

    loop {
        match listener.accept().await {
            Ok((mut socket, peer_addr)) => {
                accepted_count += 1;
                println!("  [server] Accepted connection #{accepted_count} from {peer_addr}");
                tokio::spawn(async move {
                    let mut buf = [0u8; 32];
                    if let Ok(n) = socket.read(&mut buf).await {
                        let _ = socket.write_all(&buf[..n]).await;
                    }
                });
            }
            Err(err) => {
                // In production, ECONNABORTED or EMFILE shouldn't kill your server!
                match err.kind() {
                    ErrorKind::ConnectionAborted | ErrorKind::Interrupted => {
                        eprintln!("  [server warning] Transient error: {err}. Retrying...");
                        continue;
                    }
                    ErrorKind::WouldBlock => {
                        // Sleep briefly before polling again if OS table was full
                        sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                    _ => {
                        eprintln!("  [server fatal] Non-recoverable error: {err}");
                        break;
                    }
                }
            }
        }

        if accepted_count >= 3 {
            println!("  [server] Successfully served 3 requests. Shutting down loop.");
            break;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!(" Tokio Pattern 04: Resilient TcpListener Accept Loop");
    println!("============================================================");

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    println!("▶ Bound resilient server to {addr}");

    let server = tokio::spawn(run_resilient_server(listener));

    // Client connects
    for i in 1..=3 {
        let mut client = TcpStream::connect(addr).await?;
        client.write_all(format!("ping-{i}").as_bytes()).await?;
        let mut buf = [0u8; 32];
        let n = client.read(&mut buf).await?;
        println!(
            "  [client] Received response: {}",
            String::from_utf8_lossy(&buf[..n])
        );
    }

    server.await?;
    println!("\n✔ Finished Pattern 04: Server loop withstood operations without exiting.\n");
    Ok(())
}
