use std::sync::Arc;

use crate::matcher::Matcher;

pub fn any() -> Matcher {
    Arc::new(|_| true)
}
