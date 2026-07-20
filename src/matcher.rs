use std::sync::Arc;

pub type Matcher = Arc<dyn Fn(&[u8]) -> bool + Send + Sync>;
