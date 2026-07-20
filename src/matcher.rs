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

/// Combinator: match only if **both** matchers match.
///
/// # Example
///
/// ```
/// use cmux_rs::matcher::{from_fn, and};
///
/// let long = from_fn(|p| p.len() > 4);
/// let http = from_fn(|p| p.starts_with(b"GET "));
/// let m = and(long, http);
/// assert!(m(b"GET /index.html"));
/// assert!(!m(b"GET "));
/// ```
#[must_use]
pub fn and(left: Matcher, right: Matcher) -> Matcher {
    Arc::new(move |peek| left(peek) && right(peek))
}

/// Combinator: match if **either** matcher matches.
///
/// # Example
///
/// ```
/// use cmux_rs::matcher::{from_fn, or};
///
/// let get = from_fn(|p| p.starts_with(b"GET "));
/// let post = from_fn(|p| p.starts_with(b"POST "));
/// let m = or(get, post);
/// assert!(m(b"GET /"));
/// assert!(m(b"POST /"));
/// assert!(!m(b"PUT /"));
/// ```
#[must_use]
pub fn or(left: Matcher, right: Matcher) -> Matcher {
    Arc::new(move |peek| left(peek) || right(peek))
}

/// Combinator: match if the inner matcher does **not** match.
///
/// # Example
///
/// ```
/// use cmux_rs::matcher::{from_fn, not};
///
/// let tls = from_fn(|p| p.starts_with(&[0x16, 0x03]));
/// let plaintext = not(tls);
/// assert!(plaintext(b"GET /"));
/// assert!(!plaintext(&[0x16, 0x03, 0x01]));
/// ```
#[must_use]
pub fn not(inner: Matcher) -> Matcher {
    Arc::new(move |peek| !inner(peek))
}
