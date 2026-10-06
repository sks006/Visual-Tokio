//! 02_async_io_reactor.rs
//!
//! Demonstrates how Tokio multiplexes many concurrent I/O connections
//! across worker threads using the OS reactor (epoll / kqueue / IOCP).
//! Tasks yield on WouldBlock and are only scheduled when bytes arrive.

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!(" Tokio Pattern 02: Async I/O & OS Reactor Multiplexing");
    println!("============================================================");

    // Bind on an OS-assigned ephemeral port
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let local_addr = listener.local_addr()?;
    println!("▶ Server listening on {local_addr}");

    // Spawn server accept loop in background
    let server_task = tokio::spawn(async move {
        let mut connection_count = 0;
        loop {
            tokio::select! {
                accepted = listener.accept() => {
                    let (mut socket, peer) = accepted.expect("accept failed");
                    connection_count += 1;
                    let id = connection_count;
                    tokio::spawn(async move {
                        let mut buf = [0u8; 64];
                        // Read message from client
                        let n = socket.read(&mut buf).await.expect("read failed");
                        let received = String::from_utf8_lossy(&buf[..n]);
                        // Echo response back
                        let response = format!("ECHO:{received}");
                        socket.write_all(response.as_bytes()).await.expect("write failed");
                        println!("  [server] Client #{id} ({peer}) served: {response}");
                    });
                }
                _ = sleep(Duration::from_millis(600)) => {
                    // Stop server after testing
                    break;
                }
            }
        }
    });

    // Spawn 8 concurrent clients
    println!("▶ Spawning 8 concurrent async client tasks...");
    let mut client_handles = Vec::new();

    for i in 1..=8 {
        let addr = local_addr;
        let handle = tokio::spawn(async move {
            // Stagger client connections slightly
            sleep(Duration::from_millis(i * 15)).await;
            let mut stream = TcpStream::connect(addr).await.expect("connect failed");
            let msg = format!("msg-from-{i}");
            stream
                .write_all(msg.as_bytes())
                .await
                .expect("client write failed");

            let mut reply_buf = [0u8; 64];
            let n = stream
                .read(&mut reply_buf)
                .await
                .expect("client read failed");
            let reply = String::from_utf8_lossy(&reply_buf[..n]);
            println!("  [client #{i}] Received: {reply}");
        });
        client_handles.push(handle);
    }

    // Await all client tasks
    for handle in client_handles {
        handle.await?;
    }

    server_task.await?;
    println!("\n✔ Finished Pattern 02: 8 connections multiplexed asynchronously!\n");
    Ok(())
}
