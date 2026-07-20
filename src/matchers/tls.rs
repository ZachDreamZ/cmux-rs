use std::sync::Arc;

use crate::matcher::Matcher;

/// Matches TLS connections by their record-layer handshake
/// (`0x16 0x03 0xNN ...`).
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::tls;
///
/// let m = tls();
/// assert!(m(&[0x16, 0x03, 0x03, 0x01, 0x00]));
/// assert!(m(&[0x16, 0x03, 0x01, 0x00]));
/// assert!(!m(&[0x17, 0x03, 0x03])); // not a handshake
/// assert!(!m(&[0x16, 0x02, 0x03])); // bad major version
/// assert!(!m(&[0x16]));             // too short
/// ```
#[must_use]
pub fn tls() -> Matcher {
    Arc::new(|peek: &[u8]| {
        if peek.len() < 3 {
            return false;
        }
        // TLS record layer: ContentType = 21 (handshake)
        if peek[0] != 0x16 {
            return false;
        }
        // Major version 3
        if peek[1] != 0x03 {
            return false;
        }
        // Minor version 0..=4 (SSLv3 .. TLS 1.3)
        (0x00..=0x04).contains(&peek[2])
    })
}

/// Matches TLS connections whose **`ClientHello` SNI** (server name indication)
/// matches `pred`.
///
/// This peeks into the TLS handshake to extract the requested host name and
/// applies `pred` to it. Connections that are not TLS, or whose `ClientHello`
/// cannot be parsed within the peeked window, return `false`.
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::sni;
/// use std::sync::Arc;
///
/// // Route connections whose TLS ClientHello requests "api.example.com".
/// let m = sni(Arc::new(|name: &str| name == "api.example.com"));
/// ```
#[must_use]
pub fn sni<P>(pred: Arc<P>) -> Matcher
where
    P: Fn(&str) -> bool + Send + Sync + 'static,
{
    Arc::new(move |peek: &[u8]| {
        if !tls()(peek) {
            return false;
        }
        match extract_sni(peek) {
            Some(name) => pred(&name),
            None => false,
        }
    })
}

/// Best-effort SNI extraction from a peeked TLS `ClientHello`.
///
/// Layout (simplified):
/// ```text
/// [0]       record type = 0x16
/// [1..3]    record version (ignored)
/// [3..5]    record length
/// [5]       handshake type = 0x01 (ClientHello)
/// [6..9]    handshake length (24-bit)
/// [9..14]   client version + random (skip 2 + 32)
/// ...       session_id, cipher_suites, compression
/// [exts]    extensions; SNI is type 0x0000
/// ```
fn extract_sni(peek: &[u8]) -> Option<String> {
    // Need at least the record header + handshake header.
    if peek.len() < 5 + 4 + 38 {
        return None;
    }

    let mut pos = 5; // skip record header
                     // handshake type must be ClientHello (0x01)
    if peek[pos] != 0x01 {
        return None;
    }
    pos += 4; // skip handshake type + 24-bit length
    pos += 2; // client version
    pos += 32; // random

    // session_id
    if pos >= peek.len() {
        return None;
    }
    let sid_len = peek[pos] as usize;
    pos += 1 + sid_len;

    if pos + 2 > peek.len() {
        return None;
    }
    let cs_len = u16::from_be_bytes([peek[pos], peek[pos + 1]]) as usize;
    pos += 2 + cs_len;

    if pos >= peek.len() {
        return None;
    }
    let comp_len = peek[pos] as usize;
    pos += 1 + comp_len;

    // extensions
    if pos + 2 > peek.len() {
        return None;
    }
    let ext_total = u16::from_be_bytes([peek[pos], peek[pos + 1]]) as usize;
    pos += 2;
    let ext_end = (pos + ext_total).min(peek.len());

    while pos + 4 <= ext_end {
        let ext_type = u16::from_be_bytes([peek[pos], peek[pos + 1]]);
        let ext_len = u16::from_be_bytes([peek[pos + 2], peek[pos + 3]]) as usize;
        pos += 4;
        if ext_type != 0x0000 {
            // not SNI; skip
            pos += ext_len;
            continue;
        }
        // SNI extension:
        // server_name_list_length (2) -> [name_type(1)=0 host_name_length(2) host_name]
        if pos + 2 > peek.len() {
            return None;
        }
        let _sni_list_len = u16::from_be_bytes([peek[pos], peek[pos + 1]]) as usize;
        pos += 2;
        if pos >= peek.len() {
            return None;
        }
        let name_type = peek[pos];
        pos += 1;
        if name_type != 0x00 {
            return None; // only host_name supported
        }
        if pos + 2 > peek.len() {
            return None;
        }
        let name_len = u16::from_be_bytes([peek[pos], peek[pos + 1]]) as usize;
        pos += 2;
        if pos + name_len > peek.len() {
            return None;
        }
        return std::str::from_utf8(&peek[pos..pos + name_len])
            .ok()
            .map(str::to_string);
    }

    None
}
