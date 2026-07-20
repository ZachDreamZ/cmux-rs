use cmux_rs::matchers::{any, http1_fast};
use cmux_rs::{Cmux, Matcher};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn shutdown_stops_serve() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, _http_l) = Cmux::new(listener).match_fn(http1_fast());
    let handle = mux.shutdown_handle();

    let serve_task = tokio::spawn(async move { mux.serve().await });

    // Give the server a moment to start accepting.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Signal shutdown.
    handle.signal();

    // serve() should return cleanly (not hang).
    let res = tokio::time::timeout(std::time::Duration::from_secs(2), serve_task)
        .await
        .expect("serve() did not return after shutdown")
        .unwrap();
    assert!(res.is_ok());

    // After shutdown, the listener port is released.
    let rebound = TcpListener::bind(addr).await;
    assert!(rebound.is_ok(), "port should be free after shutdown");
}

#[tokio::test]
async fn unmatched_without_any_is_dropped() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Only an HTTP matcher, no catch-all.
    let (mux, mut http_l) = Cmux::new(listener).match_fn(http1_fast());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    // Drain any http matches so the channel doesn't fill (it's unbounded, but
    // keep the task alive).
    let http_drain = tokio::spawn(async move { while let Some(_s) = http_l.accept().await {} });

    // Connect with a non-HTTP payload; it must be dropped, not routed.
    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"NOT-HTTP-DATA-XYZ").await.unwrap();

    // Give the muxer time to read + drop. If it were routed to a channel we'd
    // never know, but the connection should simply close server-side.
    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    // The server should have closed its side (write should fail or read 0).
    let mut buf = [0u8; 8];
    let n = client.read(&mut buf).await.unwrap_or(0);
    assert_eq!(n, 0, "unmatched connection should have been closed");

    http_drain.abort();
}

#[tokio::test]
async fn local_addr_matches_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let expected = listener.local_addr().unwrap();
    let (mux, _l) = Cmux::new(listener).match_fn(any());
    assert_eq!(mux.local_addr().unwrap(), expected);
}

#[tokio::test]
async fn custom_matcher_via_from_fn() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let my_proto: Matcher = Arc::new(|peek| peek.starts_with(b"MYPROTO"));
    let (mux, mut my_l) = Cmux::new(listener).match_fn(my_proto);
    let (mux, mut any_l) = mux.match_fn(any());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        while let Some(mut s) = my_l.accept().await {
            let _ = s.write_all(b"MYPROTO-OK").await;
        }
    });
    tokio::spawn(async move {
        while let Some(mut s) = any_l.accept().await {
            let _ = s.write_all(b"ANY-OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"MYPROTO handshake").await.unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"MYPROTO-OK");
}

#[tokio::test]
async fn buffered_stream_peek_shrinks_after_read() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, mut http_l) = Cmux::new(listener).match_fn(http1_fast());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        if let Some(mut s) = http_l.accept().await {
            // Peek should show the full prefix initially.
            assert!(s.peek().starts_with(b"GET /"));
            // After reading part of it, peek should shrink.
            let mut small = [0u8; 4];
            let _ = s.read(&mut small).await;
            assert_eq!(&small, b"GET ");
            assert!(s.peek().starts_with(b"/index"));
            let _ = s.write_all(b"OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client
        .write_all(b"GET /index.html HTTP/1.1\r\n\r\n")
        .await
        .unwrap();
    let mut buf = [0u8; 8];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"OK");
}

#[tokio::test]
async fn into_inner_returns_stream() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, mut http_l) = Cmux::new(listener).match_fn(http1_fast());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        if let Some(s) = http_l.accept().await {
            // Taking the inner stream should still allow writing.
            let mut stream = s.into_inner();
            let _ = stream.write_all(b"RAW-OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"RAW-OK");
}
