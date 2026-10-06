//! 05_echo_server.rs
//!
//! Demonstrates the high-performance Tokio TCP echo server pipeline:
//! Splitting a `TcpStream` into owned read and write halves with zero-copy-like piping.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn handle_client(mut socket: TcpStream) -> std::io::Result<()> {
    let (mut reader, mut writer) = socket.split();
    // tokio::io::copy handles bidirectional reading and writing asynchronously
    let bytes_copied = tokio::io::copy(&mut reader, &mut writer).await?;
    println!("  [handler] Echoed {bytes_copied} bytes to peer");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!(" Tokio Pattern 05: High-Performance TCP Echo Pipeline");
    println!("============================================================");

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    println!("▶ Echo server running on {addr}");

    let server_handle = tokio::spawn(async move {
        for _ in 0..2 {
            let (socket, _) = listener.accept().await.expect("accept failed");
            tokio::spawn(handle_client(socket));
        }
    });

    // Client 1
    let mut c1 = TcpStream::connect(addr).await?;
    c1.write_all(b"Hello Tokio!").await?;
    c1.shutdown().await?; // signals EOF
    let mut resp1 = Vec::new();
    c1.read_to_end(&mut resp1).await?;
    println!(
        "  [client 1] Got reply: {}",
        String::from_utf8_lossy(&resp1)
    );

    // Client 2
    let mut c2 = TcpStream::connect(addr).await?;
    c2.write_all(b"Async I/O rocks.").await?;
    c2.shutdown().await?;
    let mut resp2 = Vec::new();
    c2.read_to_end(&mut resp2).await?;
    println!(
        "  [client 2] Got reply: {}",
        String::from_utf8_lossy(&resp2)
    );

    server_handle.await?;
    println!("\n✔ Finished Pattern 05: Echo pipeline executed cleanly.\n");
    Ok(())
}
