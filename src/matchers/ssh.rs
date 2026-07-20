use std::sync::Arc;

use crate::matcher::Matcher;

/// Matches SSH connections by their identification string (`SSH-2.0-...`).
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::ssh;
///
/// let m = ssh();
/// assert!(m(b"SSH-2.0-OpenSSH_8.0\r\n"));
/// assert!(m(b"SSH-1.99-...\r\n"));
/// assert!(!m(b"GET / HTTP/1.1"));
/// assert!(!m(b"ssh-2.0")); // lowercase
/// ```
#[must_use]
pub fn ssh() -> Matcher {
    Arc::new(|peek: &[u8]| {
        let Ok(s) = std::str::from_utf8(peek) else {
            return false;
        };
        s.starts_with("SSH-")
    })
}
