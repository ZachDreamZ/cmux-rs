use std::sync::Arc;

use crate::matcher::Matcher;

pub fn ssh() -> Matcher {
    Arc::new(|peek: &[u8]| {
        let s = match std::str::from_utf8(peek) {
            Ok(s) => s,
            Err(_) => return false,
        };
        s.starts_with("SSH-")
    })
}
