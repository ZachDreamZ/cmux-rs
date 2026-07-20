use std::sync::Arc;

/// A protocol matcher.
///
/// Given the first bytes (`peek`) of an accepted connection, return `true` if
/// the connection belongs to the protocol this matcher detects.
///
/// Matchers are tried in registration order; the first one returning `true`
/// wins. Build one with [`from_fn`] or any of the helpers in
/// [`crate::matchers`].
///
/// # Example
///
/// ```
/// use cmux_rs::matcher::from_fn;
/// use cmux_rs::Matcher;
///
/// // Detect a bespoke "MYPROTO" handshake.
/// let m: Matcher = from_fn(|peek| peek.starts_with(b"MYPROTO"));
/// assert!(m(b"MYPROTO hello"));
/// assert!(!m(b"other"));
/// ```
pub type Matcher = Arc<dyn Fn(&[u8]) -> bool + Send + Sync>;

/// Build a [`Matcher`] from a closure.
///
/// This is a convenience around `Arc::new(closure)` that also documents the
/// intended signature.
pub fn from_fn<F>(f: F) -> Matcher
where
    F: Fn(&[u8]) -> bool + Send + Sync + 'static,
{
    Arc::new(f)
}
