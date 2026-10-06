//! integration_tests.rs
//!
//! Automated test suite validating that all Tokio patterns compile, execute,
//! and terminate cleanly within expected boundaries.

use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_visual_guide::{channel_patterns, io_patterns, resilient_listener, select_patterns};

#[tokio::test]
async fn test_select_race_timeout() {
    let fast_op = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        "fast_ok"
    };

    let res = select_patterns::race_with_timeout(fast_op, Duration::from_millis(200)).await;
    assert_eq!(res, Ok("fast_ok"));

    let slow_op = async {
        tokio::time::sleep(Duration::from_millis(250)).await;
        "slow_ok"
    };

    let res_timeout = select_patterns::race_with_timeout(slow_op, Duration::from_millis(30)).await;
    assert!(res_timeout.is_err());
}

#[tokio::test]
async fn test_resilient_listener_classification() {
    let aborted_err = std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "client aborted");
    assert!(resilient_listener::is_transient_error(&aborted_err));

    let fatal_err = std::io::Error::new(std::io::ErrorKind::AddrInUse, "address in use");
    assert!(!resilient_listener::is_transient_error(&fatal_err));
}

#[tokio::test]
async fn test_echo_stream_and_pipe() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        io_patterns::echo_stream(socket).await.unwrap();
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    client.write_all(b"tokio-test-payload").await.unwrap();
    client.shutdown().await.unwrap();

    let mut response = Vec::new();
    client.read_to_end(&mut response).await.unwrap();
    assert_eq!(&response, b"tokio-test-payload");

    server.await.unwrap();
}

#[tokio::test]
async fn test_mpsc_producer_consumer() {
    let (tx, mut rx) = mpsc::channel(4);

    let h1 = channel_patterns::spawn_producer(tx.clone(), 1, "msg-1");
    let h2 = channel_patterns::spawn_producer(tx.clone(), 2, "msg-2");
    drop(tx); // drop coordinator handle

    let mut received = Vec::new();
    while let Some(msg) = rx.recv().await {
        received.push(msg.payload);
    }

    assert_eq!(received.len(), 2);
    assert!(received.contains(&"msg-1".to_string()));
    assert!(received.contains(&"msg-2".to_string()));

    h1.await.unwrap();
    h2.await.unwrap();
}

#[tokio::test]
async fn test_arc_mutex_shared_state() {
    use std::sync::{Arc, Mutex};
    use tokio_visual_guide::state_patterns;

    let counter = Arc::new(Mutex::new(0));
    let final_count = state_patterns::increment_counter_concurrently(counter, 10).await;
    assert_eq!(final_count, 10);
}
