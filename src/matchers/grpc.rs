use std::sync::Arc;

use crate::matcher::Matcher;

const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
const GRPC_CONTENT_TYPE: &[u8] = b"content-type: application/grpc";

/// Matches HTTP/2 connections by their connection preface.
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::http2;
///
/// let m = http2();
/// assert!(m(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"));
/// assert!(!m(b"GET / HTTP/1.1"));
/// ```
#[must_use]
pub fn http2() -> Matcher {
    Arc::new(|peek: &[u8]| peek.starts_with(HTTP2_PREFACE))
}

/// Matches HTTP/2 connections by preface, or gRPC by the
/// `content-type: application/grpc` header appearing in the peeked bytes.
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::grpc;
///
/// let m = grpc();
/// assert!(m(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"));
/// assert!(m(b"\x00\x00\x00\x04\x00\x00\x00\x00\x00content-type: application/grpc"));
/// assert!(!m(b"GET / HTTP/1.1"));
/// ```
#[must_use]
pub fn grpc() -> Matcher {
    Arc::new(|peek: &[u8]| {
        peek.starts_with(HTTP2_PREFACE)
            || peek
                .windows(GRPC_CONTENT_TYPE.len())
                .any(|w| w == GRPC_CONTENT_TYPE)
    })
}
