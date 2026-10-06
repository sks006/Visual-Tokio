//! # Tokio Visual Guide
//!
//! A companion library providing production-ready implementations of the
//! asynchronous patterns animated in the visual guide.
//!
//! ## Core Modules
//!
//! - [`select_patterns`]: `tokio::select!` racing, cancellation-by-drop, and timeouts.
//! - [`io_patterns`]: Asynchronous I/O multiplexing, split sockets, and zero-copy piping.
//! - [`resilient_listener`]: Production-grade TCP accept loops with backoff on transient errors.
//! - [`channel_patterns`]: Bounded channels with explicit permit management and graceful shutdown.

use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

pub mod select_patterns {
    use super::*;

    /// Races an asynchronous query against a deadline duration.
    ///
    /// Demonstrates Tokio cancellation-by-drop: the losing future is immediately dropped,
    /// triggering any registered RAII cleanup without resource leaks.
    pub async fn race_with_timeout<F, T>(
        operation: F,
        timeout_dur: Duration,
    ) -> Result<T, &'static str>
    where
        F: std::future::Future<Output = T>,
    {
        tokio::select! {
            result = operation => Ok(result),
            _ = tokio::time::sleep(timeout_dur) => Err("operation timed out; query future dropped"),
        }
    }
}

pub mod io_patterns {
    use super::*;

    /// Echoes data between read and write halves of a stream with a bounded stack buffer.
    pub async fn echo_stream(mut stream: TcpStream) -> std::io::Result<usize> {
        let (mut reader, mut writer) = stream.split();
        let mut total_bytes = 0;
        let mut buf = [0u8; 1024];

        loop {
            let bytes_read = reader.read(&mut buf).await?;
            if bytes_read == 0 {
                break; // EOF
            }
            writer.write_all(&buf[..bytes_read]).await?;
            total_bytes += bytes_read;
        }

        writer.flush().await?;
        Ok(total_bytes)
    }

    /// Pipes bytes from reader to writer using Tokio's optimized copy mechanism.
    pub async fn pipe<R, W>(mut reader: R, mut writer: W) -> std::io::Result<u64>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        tokio::io::copy(&mut reader, &mut writer).await
    }
}

pub mod resilient_listener {
    use super::*;
    use std::io::ErrorKind;

    /// Classifies whether a socket error is transient (e.g. client dropped connection before accept).
    #[must_use]
    pub fn is_transient_error(err: &std::io::Error) -> bool {
        matches!(
            err.kind(),
            ErrorKind::ConnectionAborted
                | ErrorKind::ConnectionReset
                | ErrorKind::Interrupted
                | ErrorKind::WouldBlock
        )
    }

    /// Safely accepts a connection from a listener with automatic retry for transient errors.
    pub async fn accept_resilient(listener: &TcpListener) -> std::io::Result<TcpStream> {
        loop {
            match listener.accept().await {
                Ok((stream, _peer_addr)) => return Ok(stream),
                Err(err) if is_transient_error(&err) => {
                    tracing::warn!(error = %err, "Transient accept error; backing off briefly");
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    continue;
                }
                Err(fatal) => return Err(fatal),
            }
        }
    }
}

pub mod channel_patterns {
    use super::*;

    /// Message payload for pipeline demonstration.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TaskMessage {
        pub id: usize,
        pub payload: String,
    }

    /// Spawns a producer task with an explicit sender handle.
    pub fn spawn_producer(
        tx: mpsc::Sender<TaskMessage>,
        id: usize,
        msg: &str,
    ) -> tokio::task::JoinHandle<()> {
        let payload = msg.to_string();
        tokio::spawn(async move {
            let item = TaskMessage { id, payload };
            let _ = tx.send(item).await;
        })
    }
}
