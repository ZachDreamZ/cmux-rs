use std::sync::Arc;

use crate::matcher::Matcher;

const HTTP_METHODS: &[&str] = &[
    "GET ", "POST ", "HEAD ", "PUT ", "DELETE ", "OPTIONS ", "PATCH ", "CONNECT ", "TRACE ",
];

/// Matches HTTP/1.x requests by their method token (`GET `, `POST `, ...).
///
/// This is an optimistic, fast matcher: it only inspects the first bytes and
/// does not validate the full request line. For stricter matching, parse the
/// request line yourself with [`crate::matcher::from_fn`].
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::http1_fast;
///
/// let m = http1_fast();
/// assert!(m(b"GET / HTTP/1.1\r\n"));
/// assert!(m(b"POST /api HTTP/1.1\r\n"));
/// assert!(!m(b"SSH-2.0-OpenSSH"));
/// assert!(!m(b""));
/// ```
#[must_use]
pub fn http1_fast() -> Matcher {
    Arc::new(|peek: &[u8]| {
        let Ok(s) = std::str::from_utf8(&peek[..std::cmp::min(peek.len(), 16)]) else {
            return false;
        };
        HTTP_METHODS.iter().any(|m| s.starts_with(m))
    })
}
