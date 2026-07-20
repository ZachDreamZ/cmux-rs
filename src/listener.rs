use tokio::sync::mpsc;

use crate::buffer::BufferedStream;

pub struct MatchedListener {
    rx: mpsc::UnboundedReceiver<BufferedStream>,
}

impl MatchedListener {
    #[must_use]
    pub fn new(rx: mpsc::UnboundedReceiver<BufferedStream>) -> Self {
        MatchedListener { rx }
    }

    pub async fn accept(&mut self) -> Option<BufferedStream> {
        self.rx.recv().await
    }
}
