use std::sync::Arc;

use crate::matcher::Matcher;

const HTTP_METHODS: &[&str] = &[
    "GET ", "POST ", "HEAD ", "PUT ", "DELETE ", "OPTIONS ", "PATCH ", "CONNECT ", "TRACE ",
];

pub fn http1_fast() -> Matcher {
    Arc::new(|peek: &[u8]| {
        let s = match std::str::from_utf8(&peek[..std::cmp::min(peek.len(), 16)]) {
            Ok(s) => s,
            Err(_) => return false,
        };
        HTTP_METHODS.iter().any(|m| s.starts_with(m))
    })
}
