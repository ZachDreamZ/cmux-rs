//! `cmux-rs` — Connection multiplexer for Rust
//!
//! Serve multiple protocols on a single TCP port by inspecting the
//! first few bytes of each connection and routing to the correct handler.
//!
//! ```no_run
//! use cmux_rs::Cmux;
//! use cmux_rs::matchers::{http1_fast, tls, any};
//! use tokio::net::TcpListener;
//!
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     let listener = TcpListener::bind("127.0.0.1:8080").await?;
//!     let (mux, _http_l) = Cmux::new(listener).match_fn(http1_fast());
//!     let (mux, _tls_l) = mux.match_fn(tls());
//!     let (_mux, _any_l) = mux.match_fn(any());
//!     // serve each virtual listener in its own task, then:
//!     // mux.serve().await?;
//!     Ok(())
//! }
//! ```

mod buffer;
mod cmux;
mod error;
mod listener;
pub mod matcher;

pub mod matchers;

pub use buffer::BufferedStream;
pub use cmux::{Cmux, Shutdown};
pub use error::Error;
pub use listener::MatchedListener;
pub use matcher::{from_fn, Matcher};
