use cmux_rs::matchers::{any, grpc, http1_fast, ssh, tls};

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
