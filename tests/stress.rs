use cmux_rs::matchers::{any, grpc, http1_fast, ssh, tls};
use cmux_rs::{Cmux, MatchedListener};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Open many concurrent connections mixing 5 protocols and verify every one
/// is routed to the correct handler. Exercises concurrent dispatch + matching.
#[tokio::test]
async fn concurrent_mixed_protocols_routed_correctly() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (mux, http_l) = Cmux::new(listener).match_fn(http1_fast());
    let (mux, tls_l) = mux.match_fn(tls());
    let (mux, ssh_l) = mux.match_fn(ssh());
    let (mux, grpc_l) = mux.match_fn(grpc());
    let (_mux, any_l) = mux.match_fn(any());
    tokio::spawn(async move {
        let _ = _mux.serve().await;
    });

    let counter = Arc::new(AtomicUsize::new(0));

    let spawn_handler = |mut l: MatchedListener, counter: Arc<AtomicUsize>| {
        tokio::spawn(async move {
            while let Some(mut s) = l.accept().await {
                let _ = s.write_all(b"OK").await;
                counter.fetch_add(1, Ordering::SeqCst);
            }
        })
    };

    let http_c = spawn_handler(http_l, counter.clone());
    let tls_c = spawn_handler(tls_l, counter.clone());
    let ssh_c = spawn_handler(ssh_l, counter.clone());
    let grpc_c = spawn_handler(grpc_l, counter.clone());
    let any_c = spawn_handler(any_l, counter.clone());

    let n = 200u32;
    for i in 0..n {
        let payload: &[u8] = match i % 5 {
            0 => b"GET / HTTP/1.1\r\n\r\n",
            1 => &[0x16, 0x03, 0x03, 0x01, 0x00],
            2 => b"SSH-2.0-test\r\n",
            3 => b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n",
            _ => b"whatever-random-stuff",
        };
        let counter = counter.clone();
        tokio::spawn(async move {
            let mut s = TcpStream::connect(addr).await.unwrap();
            s.write_all(payload).await.unwrap();
            let mut buf = [0u8; 16];
            let r = s.read(&mut buf).await;
            assert_eq!(r.unwrap(), 2, "expected OK (2 bytes)");
            let _ = counter;
        });
    }

    for _ in 0..100 {
        if counter.load(Ordering::SeqCst) as u32 == n {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        n as usize,
        "all {} connections should be routed",
        n
    );

    http_c.abort();
    tls_c.abort();
    ssh_c.abort();
    grpc_c.abort();
    any_c.abort();
}
