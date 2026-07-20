//! Serve HTTP/1.1, HTTP/2 + gRPC, SSH, and a catch-all on the same port.
//!
//! Run with: `cargo run --example multi`

use cmux_rs::matchers::{any, grpc, http1_fast, ssh};
use cmux_rs::Cmux;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("cmux-rs multi example listening on :8080");

    let (mux, mut http_l) = Cmux::new(listener).match_fn(http1_fast());
    let (mux, mut grpc_l) = mux.match_fn(grpc());
    let (mux, mut ssh_l) = mux.match_fn(ssh());
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
        while let Some(mut s) = grpc_l.accept().await {
            let _ = s.write_all(b"HTTP/2 / gRPC connection detected\n").await;
        }
    });

    tokio::spawn(async move {
        while let Some(mut s) = ssh_l.accept().await {
            let _ = s.write_all(b"SSH connection detected\n").await;
        }
    });

    tokio::spawn(async move {
        while let Some(mut s) = any_l.accept().await {
            let _ = s.write_all(b"unknown protocol\n").await;
        }
    });

    mux.serve().await
}
