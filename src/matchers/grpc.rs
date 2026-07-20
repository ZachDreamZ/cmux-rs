use std::sync::Arc;

use crate::matcher::Matcher;

const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub fn http2() -> Matcher {
    Arc::new(|peek: &[u8]| peek.starts_with(HTTP2_PREFACE))
}

pub fn grpc() -> Matcher {
    Arc::new(|peek: &[u8]| {
        peek.starts_with(HTTP2_PREFACE)
            || peek.windows(20).any(|w| {
                w.windows(17)
                    .any(|s| s.starts_with(b"content-type: application/grpc"))
            })
    })
}
