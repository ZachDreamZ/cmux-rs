# Architecture

`cmux-rs` is a connection multiplexer for Rust. It lets a single TCP listener
serve multiple protocols (HTTP/1.1, HTTPS/TLS, SSH, HTTP/2, gRPC, and more) by
inspecting the first bytes of each incoming connection and routing it to the
correct handler — all without a proxy, sidecar, or extra port.

This document explains how it works and the design decisions behind it.

---

## The Problem

Normally, each network service needs its own port:

| Service        | Port |
| -------------- | ---- |
| HTTP API       | 8080 |
| HTTPS / TLS    | 8443 |
| gRPC           | 9090 |
| SSH            | 22   |

But clients often only have **one** port open (e.g. through a firewall, a
load balancer, or a cloud ingress). `cmux-rs` solves this by listening once and
deciding where each connection belongs at runtime.

This is the same idea as Go's [`cmux`](https://github.com/soheilhy/cmux), but
designed from the ground up for async Rust + Tokio.

---

## How It Works

```
                ┌─────────────────────────────────────────────┐
   client  ───▶ │                  cmux-rs                     │
                │                                              │
                │   accept()  ──▶  peek first N bytes           │
                │                       │                      │
                │                       ▼                      │
                │            try matchers in order             │
                │            (first match wins)                 │
                │              /      |       \                │
                │            /        |        \               │
                ▼          ▼         ▼         ▼              │
           HTTP/1.1     HTTPS      SSH       gRPC   ...        │
           listener     listener   listener   listener         │
                │          │         │         │               │
                ▼          ▼         ▼         ▼               │
            your app    your app  your app  your app           │
                └─────────────────────────────────────────────┘
```

### 1. Accept

`Cmux` wraps a `tokio::net::TcpListener`. For each accepted connection it
spawns a lightweight task.

### 2. Peek

The task reads up to `peek_len` bytes (default **4096**) from the stream but
does **not** consume them. The bytes are stored in a prefix buffer.

A short timeout (`500ms`) guards against clients that connect but send nothing.

### 3. Match

Each registered matcher is a `fn(&[u8]) -> bool`. Matchers run **in order** of
registration; the **first** one that returns `true` wins.

| Matcher        | Detects                                       |
| -------------- | --------------------------------------------- |
| `http1_fast`   | HTTP method (`GET `, `POST `, ...)            |
| `tls`          | TLS handshake record (`0x16 0x03 0xNN`)      |
| `ssh`          | `SSH-` identification string                  |
| `http2` / `grpc` | HTTP/2 preface / `content-type: application/grpc` |
| `any`          | catch-all (always matches)                    |

### 4. Dispatch

The connection, wrapped in a `BufferedStream` that still holds the peeked
prefix, is sent over an unbounded channel to the virtual listener that owns the
matching matcher.

### 5. Serve

The application calls `MatchedListener::accept()` (an async method that
resolves to the next matched `BufferedStream`). The handler reads from this
stream as if it were a normal `TcpStream` — the peeked bytes are replayed first,
so no data is lost.

---

## Core Types

### `Cmux`

```rust
pub struct Cmux {
    listener: TcpListener,
    matchers: Vec<(Matcher, mpsc::UnboundedSender<BufferedStream>)>,
    peek_len: usize,
}
```

The builder-style entry point:

```rust
let (mux, http_l)   = Cmux::new(listener).match_fn(http1_fast());
let (mux, tls_l)    = mux.match_fn(tls());
let (_mux, any_l)   = mux.match_fn(any());
// later:
mux.serve().await?;
```

`match_fn` returns `(Self, MatchedListener)` so you can chain registrations
and keep a handle to each virtual listener.

### `Matcher`

A matcher is just:

```rust
pub type Matcher = Arc<dyn Fn(&[u8]) -> bool + Send + Sync>;
```

This makes it trivial to write custom matchers inline:

```rust
let custom = Arc::new(|peek: &[u8]| peek.starts_with(b"MYPROTO"));
let (mux, my_l) = mux.match_fn(custom);
```

### `BufferedStream`

```rust
pub struct BufferedStream {
    stream: TcpStream,
    prefix: Vec<u8>,
    prefix_pos: usize,
}
```

It implements `tokio::io::AsyncRead` and `AsyncWrite`:

- `AsyncRead` first drains `prefix[prefix_pos..]`, then delegates to the inner
  `TcpStream`. This guarantees the peeked bytes are delivered to the handler.
- `AsyncWrite` is a straight pass-through to the inner stream.

### `MatchedListener`

```rust
pub struct MatchedListener {
    rx: mpsc::UnboundedReceiver<BufferedStream>,
}

impl MatchedListener {
    pub async fn accept(&mut self) -> Option<BufferedStream> { ... }
}
```

One virtual listener per matcher. Hand your app a `MatchedListener` and call
`accept()` in a loop (typically inside `tokio::spawn`).

---

## Design Decisions

### Why peek-and-replay instead of `try_read`?

Windows and Unix behave differently for half-open connections and read
timeouts. A peek buffer with a single bounded `read` + timeout works uniformly
across both platforms and matches the Go `cmux` approach.

### Why unbounded channels?

`Cmux` dispatches accepted connections as fast as possible. Back-pressure is the
responsibility of the application's handler tasks, not the multiplexer. Using
`UnboundedSender` keeps the accept loop non-blocking.

### Why `Arc<dyn Fn>` for matchers?

Closures and function pointers both work, matching Go's `type Matcher func()`.
No trait-object boxing overhead beyond the single `Arc`.

### Order matters

Matchers are tried in registration order. Put more specific matchers first.
`any()` should always be last — it is the catch-all.

---

## Limitations

- **One protocol per connection.** A connection is classified once, at accept
  time, by its first bytes. You cannot switch protocols mid-stream.
- **Peek length.** If a protocol's identifying bytes arrive after `peek_len`
  bytes (unlikely for the built-in matchers), raise `set_peek_len`.
- **TLS introspection.** `cmux-rs` matches the TLS *record layer*, not the
  decrypted payload. To inspect inside TLS you must terminate TLS before the
  multiplexer (or use a SNI-based matcher).

---

## Comparison with Go `cmux`

| Feature                | Go `cmux`        | `cmux-rs`              |
| ---------------------- | ---------------- | ---------------------- |
| Language               | Go               | Rust (async/Tokio)     |
| Matcher type           | `func(io.Reader)`| `fn(&[u8]) -> bool`    |
| Virtual listener       | `net.Listener`   | `MatchedListener`      |
| Built-in matchers      | HTTP1/2, TLS, SSH, gRPC | HTTP1/2, TLS, SSH, gRPC, `any` |
| Windows support        | Yes              | Yes (native)           |
| Peek mechanism         | `bufio.Reader`   | prefix buffer + replay |
