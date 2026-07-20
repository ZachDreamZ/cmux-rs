//! Serve HTTP/1.1 and TLS on the same port.
//!
//! Run with: `cargo run --example simple`

use cmux_rs::matchers::{any, http1_fast, tls};
use cmux_rs::Cmux;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("cmux-rs simple example listening on :8080");

    let (mux, mut http_l) = Cmux::new(listener).match_fn(http1_fast());
    let (mux, mut tls_l) = mux.match_fn(tls());
    let (mux, mut any_l) = mux.match_fn(any());

    tokio::spawn(async move {
        while let Some(mut s) = http_l.accept().await {
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf).await;
            let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
            let _ = s.write_all(resp).await;
        }
    });

    tokio::spawn(async move {
        while let Some(mut s) = tls_l.accept().await {
            let _ = s.write_all(b"TLS connection detected\n").await;
        }
    });

    tokio::spawn(async move {
        while let Some(mut s) = any_l.accept().await {
            let _ = s.write_all(b"other connection\n").await;
        }
    });

    mux.serve().await
}
