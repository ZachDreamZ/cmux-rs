use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch};

use crate::buffer::BufferedStream;
use crate::error::Error;
use crate::listener::MatchedListener;
use crate::matcher::Matcher;

const DEFAULT_PEEK_LEN: usize = 4096;
const PEEK_TIMEOUT: Duration = Duration::from_millis(500);

type MatcherEntry = (Matcher, mpsc::UnboundedSender<BufferedStream>);

/// A connection multiplexer.
///
/// Wrap a [`TcpListener`] and register one or more matchers. Each accepted
/// connection is peeked, matched against the registered matchers (in order),
/// and forwarded to the corresponding virtual listener.
pub struct Cmux {
    listener: TcpListener,
    matchers: Vec<MatcherEntry>,
    peek_len: usize,
    shutdown_rx: watch::Receiver<bool>,
    shutdown_tx: watch::Sender<bool>,
}

impl Cmux {
    /// Create a new multiplexer around `listener`.
    pub fn new(listener: TcpListener) -> Self {
        let (tx, rx) = watch::channel(false);
        Cmux {
            listener,
            matchers: Vec::new(),
            peek_len: DEFAULT_PEEK_LEN,
            shutdown_rx: rx,
            shutdown_tx: tx,
        }
    }

    /// Register a matcher and obtain its virtual listener.
    ///
    /// Matchers are tried in registration order; the first one that returns
    /// `true` wins. Put the most specific matchers first and `any()` last.
    pub fn match_fn(mut self, matcher: Matcher) -> (Self, MatchedListener) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.matchers.push((matcher, tx));
        (self, MatchedListener::new(rx))
    }

    /// Set the number of bytes peeked from each connection for matching.
    /// Defaults to 4096. Must be large enough to contain the identifying
    /// bytes of every matcher you register.
    #[must_use]
    pub fn set_peek_len(mut self, len: usize) -> Self {
        self.peek_len = len;
        self
    }

    /// The local address this multiplexer is listening on.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error if the listener's local address cannot
    /// be determined.
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Obtain a cloneable handle that shuts the multiplexer down.
    ///
    /// Calling [`Shutdown::signal`] causes [`serve`](Self::serve) to stop
    /// accepting new connections and return `Ok(())` once the current
    /// `accept` future resolves.
    pub fn shutdown_handle(&self) -> Shutdown {
        Shutdown {
            tx: self.shutdown_tx.clone(),
        }
    }

    /// Serve connections until the listener is closed or a shutdown is
    /// signalled via [`shutdown_handle`](Self::shutdown_handle).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Accept`] if the underlying listener fails. Transient
    /// accept errors are logged and retried; the future only resolves with an
    /// error when the listener is otherwise unusable. A `Shutdown` signal
    /// causes it to return `Ok(())`.
    pub async fn serve(mut self) -> Result<(), Error> {
        // Share the matcher table across all dispatch tasks without cloning
        // the Vec per connection — an Arc clone is a single atomic op.
        let matchers = Arc::new(self.matchers.clone());
        loop {
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    if *self.shutdown_rx.borrow() {
                        break;
                    }
                }
                accepted = self.listener.accept() => {
                    let (stream, _) = match accepted {
                        Ok(c) => c,
                        Err(e) => {
                            // A transient accept error (e.g. EMFILE) should not
                            // kill the whole multiplexer. Log and keep going.
                            tracing::warn!("accept failed: {e}");
                            continue;
                        }
                    };

                    let peek_len = self.peek_len;
                    let matchers = Arc::clone(&matchers);
                    let mut stream = stream;

                    tokio::spawn(async move {
                        let mut buf = vec![0u8; peek_len];
                        let Ok(Ok(n)) =
                            tokio::time::timeout(PEEK_TIMEOUT, stream.read(&mut buf)).await
                        else {
                            return; // timeout or read error: drop
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
                                if matchers[idx].1.send(buffered).is_err() {
                                    tracing::debug!("matcher receiver dropped; closing connection");
                                }
                            }
                            None => drop(buffered),
                        }
                    });
                }
            }
        }
        Ok(())
    }
}

/// A handle that shuts down a [`Cmux`] when signalled.
#[derive(Clone)]
pub struct Shutdown {
    tx: watch::Sender<bool>,
}

impl Shutdown {
    /// Signal the associated multiplexer to stop serving.
    pub fn signal(&self) {
        let _ = self.tx.send(true);
    }
}
