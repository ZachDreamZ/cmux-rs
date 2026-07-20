use std::fmt;

/// Errors produced by `cmux-rs`.
#[derive(Debug)]
pub enum Error {
    /// The underlying listener failed to accept a connection.
    Accept(tokio::io::Error),
    /// A connection was accepted but matched no registered matcher and no
    /// catch-all (`any()`) was registered.
    NotMatched,
    /// The multiplexer was shut down (e.g. via `Shutdown` signal) before the
    /// connection could be dispatched.
    Shutdown,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Accept(e) => write!(f, "accept error: {e}"),
            Error::NotMatched => write!(f, "connection matched no matcher"),
            Error::Shutdown => write!(f, "multiplexer is shutting down"),
        }
    }
}

impl std::error::Error for Error {}

impl From<tokio::io::Error> for Error {
    fn from(e: tokio::io::Error) -> Self {
        Error::Accept(e)
    }
}
