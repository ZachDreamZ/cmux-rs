use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::buffer::BufferedStream;
use crate::listener::MatchedListener;
use crate::matcher::Matcher;

const DEFAULT_PEEK_LEN: usize = 4096;
const PEEK_TIMEOUT: Duration = Duration::from_millis(500);

pub struct Cmux {
    listener: TcpListener,
    matchers: Vec<(Matcher, mpsc::UnboundedSender<BufferedStream>)>,
    peek_len: usize,
}

impl Cmux {
    pub fn new(listener: TcpListener) -> Self {
        Cmux {
            listener,
            matchers: Vec::new(),
            peek_len: DEFAULT_PEEK_LEN,
        }
    }

    pub fn set_peek_len(&mut self, len: usize) -> &mut Self {
        self.peek_len = len;
        self
    }

    pub fn match_fn(mut self, matcher: Matcher) -> (Self, MatchedListener) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.matchers.push((matcher, tx));
        (self, MatchedListener::new(rx))
    }

    pub async fn serve(self) -> Result<(), tokio::io::Error> {
        while let Ok((stream, _)) = self.listener.accept().await {
            let peek_len = self.peek_len;
            let matchers = self.matchers.clone();
            let mut stream = stream;

            tokio::spawn(async move {
                let mut buf = vec![0u8; peek_len];
                let n = match tokio::time::timeout(PEEK_TIMEOUT, stream.read(&mut buf)).await {
                    Ok(Ok(n)) => n,
                    _ => return,
                };
                buf.truncate(n);

                let matched = matchers
                    .iter()
                    .enumerate()
                    .find(|(_, (matcher, _))| matcher(&buf))
                    .map(|(i, _)| i);

                let buffered = BufferedStream::from_parts(stream, buf);

                match matched {
                    Some(idx) => {
                        let _ = matchers[idx].1.send(buffered);
                    }
                    None => drop(buffered),
                }
            });
        }
        Ok(())
    }
}
