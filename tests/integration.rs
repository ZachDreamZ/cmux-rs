use cmux_rs::matchers::{any, grpc, http1_fast, ssh, tls};
use cmux_rs::Cmux;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn http_routes_to_http_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, mut http_l) = Cmux::new(listener).match_fn(http1_fast());
    let (mux, mut any_l) = mux.match_fn(any());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        while let Some(mut s) = http_l.accept().await {
            let _ = s.write_all(b"HTTP-OK").await;
        }
    });
    tokio::spawn(async move {
        while let Some(mut s) = any_l.accept().await {
            let _ = s.write_all(b"ANY-OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"HTTP-OK");
}

#[tokio::test]
async fn tls_routes_to_tls_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, mut tls_l) = Cmux::new(listener).match_fn(tls());
    let (mux, mut any_l) = mux.match_fn(any());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        while let Some(mut s) = tls_l.accept().await {
            let _ = s.write_all(b"TLS-OK").await;
        }
    });
    tokio::spawn(async move {
        while let Some(mut s) = any_l.accept().await {
            let _ = s.write_all(b"ANY-OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    // TLS handshake record: 0x16 0x03 0x03 ...
    client
        .write_all(&[0x16, 0x03, 0x03, 0x01, 0x00])
        .await
        .unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"TLS-OK");
}

#[tokio::test]
async fn ssh_routes_to_ssh_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, mut ssh_l) = Cmux::new(listener).match_fn(ssh());
    let (mux, mut any_l) = mux.match_fn(any());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        while let Some(mut s) = ssh_l.accept().await {
            let _ = s.write_all(b"SSH-OK").await;
        }
    });
    tokio::spawn(async move {
        while let Some(mut s) = any_l.accept().await {
            let _ = s.write_all(b"ANY-OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"SSH-2.0-OpenSSH_8.0\r\n").await.unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"SSH-OK");
}

#[tokio::test]
async fn grpc_routes_to_grpc_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, mut grpc_l) = Cmux::new(listener).match_fn(grpc());
    let (mux, mut any_l) = mux.match_fn(any());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        while let Some(mut s) = grpc_l.accept().await {
            let _ = s.write_all(b"GRPC-OK").await;
        }
    });
    tokio::spawn(async move {
        while let Some(mut s) = any_l.accept().await {
            let _ = s.write_all(b"ANY-OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client
        .write_all(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n")
        .await
        .unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"GRPC-OK");
}

#[tokio::test]
async fn order_matters_first_match_wins() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Register TLS first, then http1 -- but TLS won't match HTTP traffic
    let (mux, mut tls_l) = Cmux::new(listener).match_fn(tls());
    let (mux, mut http_l) = mux.match_fn(http1_fast());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        while let Some(mut s) = tls_l.accept().await {
            let _ = s.write_all(b"TLS-OK").await;
        }
    });
    tokio::spawn(async move {
        while let Some(mut s) = http_l.accept().await {
            let _ = s.write_all(b"HTTP-OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"HTTP-OK");
}

#[tokio::test]
async fn any_catchall_gets_unmatched() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, mut http_l) = Cmux::new(listener).match_fn(http1_fast());
    let (mux, mut any_l) = mux.match_fn(any());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        while let Some(mut s) = http_l.accept().await {
            let _ = s.write_all(b"HTTP-OK").await;
        }
    });
    tokio::spawn(async move {
        while let Some(mut s) = any_l.accept().await {
            let _ = s.write_all(b"ANY-OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client.write_all(b"GARBAGE-DATA-12345").await.unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"ANY-OK");
}

#[tokio::test]
async fn peeked_bytes_are_preserved() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, mut http_l) = Cmux::new(listener).match_fn(http1_fast());
    let (mux, _any_l) = mux.match_fn(any());
    tokio::spawn(async move {
        let _ = mux.serve().await;
    });

    tokio::spawn(async move {
        while let Some(mut s) = http_l.accept().await {
            let mut buf = [0u8; 1024];
            let n = s.read(&mut buf).await.unwrap();
            // The peeked bytes must still be present
            assert!(std::str::from_utf8(&buf[..n]).unwrap().starts_with("GET /"));
            let _ = s.write_all(b"OK").await;
        }
    });

    let mut client = TcpStream::connect(addr).await.unwrap();
    client
        .write_all(b"GET /index.html HTTP/1.1\r\n\r\n")
        .await
        .unwrap();
    let mut buf = [0u8; 16];
    let n = client.read(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], b"OK");
}
