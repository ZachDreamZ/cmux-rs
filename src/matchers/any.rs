use std::sync::Arc;

use crate::matcher::Matcher;

/// A catch-all matcher that matches every connection.
///
/// Register this **last** so it only receives connections not matched by any
/// more specific matcher.
///
/// # Example
///
/// ```
/// use cmux_rs::matchers::any;
///
/// let m = any();
/// assert!(m(b""));
/// assert!(m(b"anything at all"));
/// assert!(m(&[0u8; 64]));
/// ```
#[must_use]
pub fn any() -> Matcher {
    Arc::new(|_| true)
}
