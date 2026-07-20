use std::sync::Arc;

use crate::matcher::Matcher;

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
