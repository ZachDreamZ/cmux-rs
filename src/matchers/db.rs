use std::sync::Arc;

use crate::matcher::Matcher;

/// Matches Redis connections (RESP protocol).
///
/// Detects both the RESP array framing used by clients (`*<n>\r\n`) and the
/// bare-text command form (`PING`, `GET `, `SET `, `AUTH`, `COMMAND`, ...).
/// Best-effort: some exotic clients may not match.
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::redis;
///
/// let m = redis();
/// assert!(m(b"*1\r\n$4\r\nPING\r\n"));
/// assert!(m(b"PING\r\n"));
/// assert!(m(b"SET mykey value\r\n"));
/// assert!(!m(b"GET / HTTP/1.1"));
/// ```
#[must_use]
pub fn redis() -> Matcher {
    Arc::new(|peek: &[u8]| {
        let mut start = 0;
        while start < peek.len()
            && (peek[start] == b' '
                || peek[start] == b'\t'
                || peek[start] == b'\r'
                || peek[start] == b'\n')
        {
            start += 1;
        }
        let trimmed = &peek[start..];
        if trimmed.first() == Some(&b'*') {
            return true; // RESP array framing
        }
        const VERBS: [&[u8]; 6] = [
            b"PING", b"SET ", b"AUTH", b"COMMAND", b"INFO", b"SELECT",
        ];
        for v in VERBS {
            if trimmed.len() >= v.len() && trimmed[..v.len()].eq_ignore_ascii_case(v) {
                return true;
            }
        }
        false
    })
}

/// Matches PostgreSQL connections by their startup message.
///
/// A PostgreSQL client opens with a 4-byte message length followed by the
/// 4-byte protocol version `0x00_03_00_00` (196608). This is highly reliable.
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::postgres;
///
/// let m = postgres();
/// let mut pkt = [0u8; 8];
/// pkt[4..8].copy_from_slice(&[0x00, 0x03, 0x00, 0x00]);
/// assert!(m(&pkt));
/// assert!(!m(b"GET / HTTP/1.1"));
/// ```
#[must_use]
pub fn postgres() -> Matcher {
    Arc::new(|peek: &[u8]| peek.len() >= 8 && peek[4..8] == [0x00, 0x03, 0x00, 0x00])
}

/// Matches MySQL / MariaDB connections.
///
/// The server greeting packet starts with the protocol-version byte `0x0A`
/// for essentially all modern MySQL/MariaDB servers. Best-effort detection of
/// the client side is not reliable, so this matches the common server-greeting
/// first byte.
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::mysql;
///
/// let m = mysql();
/// assert!(m(&[0x0a, b'5', b'.', b'7']));
/// assert!(!m(b"GET / HTTP/1.1"));
/// ```
#[must_use]
pub fn mysql() -> Matcher {
    Arc::new(|peek: &[u8]| peek.first() == Some(&0x0a))
}
