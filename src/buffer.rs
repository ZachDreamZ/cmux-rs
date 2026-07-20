use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

pub struct BufferedStream {
    stream: TcpStream,
    prefix: Vec<u8>,
    prefix_pos: usize,
}

impl BufferedStream {
    pub fn new(stream: TcpStream) -> Self {
        BufferedStream {
            stream,
            prefix: Vec::new(),
            prefix_pos: 0,
        }
    }

    pub fn from_parts(stream: TcpStream, prefix: Vec<u8>) -> Self {
        BufferedStream {
            stream,
            prefix,
            prefix_pos: 0,
        }
    }

    pub fn peek(&self) -> &[u8] {
        &self.prefix
    }

    pub fn inner(&self) -> &TcpStream {
        &self.stream
    }

    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}

impl AsyncRead for BufferedStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        if self.prefix_pos < self.prefix.len() {
            let remaining = &self.prefix[self.prefix_pos..];
            let len = std::cmp::min(remaining.len(), buf.remaining());
            buf.put_slice(&remaining[..len]);
            self.prefix_pos += len;
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for BufferedStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, tokio::io::Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), tokio::io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), tokio::io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
