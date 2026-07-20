use cmux_rs::matchers::{any, db, grpc, http1_fast, sni, ssh, tls};
use std::sync::Arc;

/// Build a minimal TLS 1.2 ClientHello requesting `sni` as the host name.
fn client_hello(sni: &str) -> Vec<u8> {
    let name_bytes = sni.as_bytes();
    let mut body = Vec::new();
    body.extend_from_slice(&[0x03, 0x03]); // client version
    body.extend_from_slice(&[0u8; 32]); // random
    body.push(0x00); // session id length 0
    body.extend_from_slice(&[0x00, 0x02]); // cipher suites length
    body.extend_from_slice(&[0x13, 0x01]); // one suite
    body.push(0x01); // compression methods length
    body.push(0x00); // null compression

    // SNI extension
    let mut sni_ext = Vec::new();
    sni_ext.extend_from_slice(&[0x00, 0x00]); // type SNI
    let sni_inner_len = 2 + 1 + 2 + name_bytes.len();
    let sni_ext_len = 2 + sni_inner_len;
    sni_ext.extend_from_slice(&(sni_ext_len as u16).to_be_bytes());
    sni_ext.extend_from_slice(&(sni_inner_len as u16).to_be_bytes()); // server_name_list length
    sni_ext.push(0x00); // name_type = host_name
    sni_ext.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
    sni_ext.extend_from_slice(name_bytes);

    let mut exts = Vec::new();
    exts.extend_from_slice(&(sni_ext.len() as u16).to_be_bytes());
    exts.extend_from_slice(&sni_ext);

    // ClientHello handshake body
    let mut ch = Vec::new();
    ch.extend_from_slice(&body);
    ch.extend_from_slice(&exts);

    // Handshake header: type ClientHello(0x01) + 24-bit length
    let mut handshake = Vec::new();
    handshake.push(0x01);
    handshake.extend_from_slice(&[
        (ch.len() >> 16) as u8,
        (ch.len() >> 8) as u8,
        ch.len() as u8,
    ]);
    handshake.extend_from_slice(&ch);

    // Record header: type handshake(0x16) + version(0x03 0x03) + 16-bit length
    let mut record = Vec::new();
    record.push(0x16);
    record.extend_from_slice(&[0x03, 0x03]);
    record.extend_from_slice(&(handshake.len() as u16).to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

#[test]
fn http1_fast_matches_methods() {
    for m in [
        "GET ", "POST ", "PUT ", "DELETE ", "HEAD ", "OPTIONS ", "PATCH ", "CONNECT ", "TRACE ",
    ] {
        assert!(http1_fast()(format!("{m}/ HTTP/1.1").as_bytes()));
    }
}

#[test]
fn http1_fast_rejects_non_http() {
    assert!(!http1_fast()(b"SSH-2.0"));
    assert!(!http1_fast()(b"\x16\x03\x03"));
    assert!(!http1_fast()(b"garbage"));
    assert!(!http1_fast()(b""));
}

#[test]
fn tls_matches_handshake() {
    assert!(tls()(&[0x16, 0x03, 0x03, 0x00]));
    assert!(tls()(&[0x16, 0x03, 0x01, 0x00]));
}

#[test]
fn tls_rejects_non_handshake() {
    assert!(!tls()(&[0x17, 0x03, 0x03])); // not handshake
    assert!(!tls()(&[0x16, 0x02, 0x03])); // bad major
    assert!(!tls()(&[0x16, 0x03, 0x99])); // bad minor
    assert!(!tls()(&[0x16])); // too short
}

#[test]
fn ssh_matches_prefix() {
    assert!(ssh()(b"SSH-2.0-OpenSSH_8.0\r\n"));
    assert!(ssh()(b"SSH-1.99-..."));
}

#[test]
fn ssh_rejects_other() {
    assert!(!ssh()(b"GET / HTTP"));
    assert!(!ssh()(b"ssh-2.0")); // lowercase
}

#[test]
fn grpc_matches_preface() {
    assert!(grpc()(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"));
}

#[test]
fn grpc_matches_content_type() {
    let frame = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\ncontent-type: application/grpc";
    assert!(grpc()(frame));
}

#[test]
fn grpc_rejects_other() {
    assert!(!grpc()(b"GET / HTTP/1.1\r\n\r\n"));
    assert!(!grpc()(b"TLS-HANDSHAKE"));
}

#[test]
fn any_matches_everything() {
    assert!(any()(b""));
    assert!(any()(b"anything at all"));
    assert!(any()(&[0u8; 64]));
}

#[test]
fn sni_matches_by_name() {
    let ch = client_hello("api.example.com");
    let m = sni(Arc::new(|n: &str| n == "api.example.com"));
    assert!(m(&ch));

    let m_other = sni(Arc::new(|n: &str| n == "other.com"));
    assert!(!m_other(&ch));
}

#[test]
fn sni_rejects_non_tls() {
    let m = sni(Arc::new(|_: &str| true));
    assert!(!m(b"GET / HTTP"));
}

#[test]
fn sni_handles_unparseable() {
    // TLS record but truncated ClientHello (no extensions).
    let truncated = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
    let m = sni(Arc::new(|_: &str| true));
    assert!(!m(&truncated));
}

#[test]
fn redis_matches_resp_array() {
    let m = db::redis();
    assert!(m(b"*1\r\n$4\r\nPING\r\n"));
}

#[test]
fn redis_matches_verbose_command() {
    let m = db::redis();
    assert!(m(b"PING\r\n"));
    assert!(m(b"set foo bar\r\n")); // case-insensitive verb
    assert!(m(b"  AUTH mypass\r\n"));
}

#[test]
fn redis_rejects_non_redis() {
    let m = db::redis();
    assert!(!m(b"GET / HTTP/1.1"));
}

#[test]
fn postgres_matches_startup() {
    let m = db::postgres();
    let mut pkt = [0u8; 8];
    pkt[4..8].copy_from_slice(&[0x00, 0x03, 0x00, 0x00]);
    assert!(m(&pkt));
}

#[test]
fn postgres_rejects_other() {
    let m = db::postgres();
    assert!(!m(b"GET / HTTP/1.1"));
}

#[test]
fn mysql_matches_greeting() {
    let m = db::mysql();
    assert!(m(&[0x0a, b'5', b'.', b'7']));
}

#[test]
fn mysql_rejects_other() {
    let m = db::mysql();
    assert!(!m(b"GET / HTTP/1.1"));
}
