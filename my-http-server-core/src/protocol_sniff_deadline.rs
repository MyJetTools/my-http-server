//! A deadline on the first bytes of a connection served by hyper-util's auto (h1/h2) builder.
//!
//! Before the auto builder hands a connection to hyper, it reads the first bytes itself to tell
//! HTTP/2 from HTTP/1: up to the 24 bytes of the HTTP/2 connection preface, or to the first byte
//! that is not the preface's (`read_version` in `hyper_util::server::conn::auto`). There is no
//! timeout on that read, and hyper's `header_read_timeout` only starts once it is over. So a client
//! that connects and sends nothing - or sends `P` and stops - would hold its connection, and its
//! file descriptor, forever, whatever the header read timeout.
//!
//! [`ProtocolSniffDeadline`] puts the header read timeout on exactly that phase: it follows the
//! same bytes the auto builder does, and once they settle the protocol it steps aside for good.
//! What comes after is hyper's: for HTTP/1 the header read timeout itself; HTTP/2 has none.

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// The HTTP/2 connection preface (RFC 9113 §3.4) - what the auto builder looks for.
const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// A stream which fails a read with [`io::ErrorKind::TimedOut`] once its deadline has passed - but
/// only until the bytes read off it have settled whether it is HTTP/2: they have strayed from the
/// HTTP/2 preface, or all of it has come. From then on it just passes reads and writes through.
pub(crate) struct ProtocolSniffDeadline<S> {
    stream: S,
    sniffing: Option<Sniffing>,
}

struct Sniffing {
    deadline: Pin<Box<tokio::time::Sleep>>,
    /// How many bytes of [`H2_PREFACE`] have come so far.
    preface_matched: usize,
}

impl<S> ProtocolSniffDeadline<S> {
    /// `timeout` counts from now. `None` puts no deadline on the stream at all.
    pub(crate) fn new(stream: S, timeout: Option<Duration>) -> Self {
        Self {
            stream,
            sniffing: timeout.map(|timeout| Sniffing {
                deadline: Box::pin(tokio::time::sleep(timeout)),
                preface_matched: 0,
            }),
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for ProtocolSniffDeadline<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        let Some(sniffing) = this.sniffing.as_mut() else {
            return Pin::new(&mut this.stream).poll_read(cx, buf);
        };

        let filled_before = buf.filled().len();

        match Pin::new(&mut this.stream).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                let read = &buf.filled()[filled_before..];
                let rest_of_preface = &H2_PREFACE[sniffing.preface_matched..];
                let len = read.len().min(rest_of_preface.len());

                // An end of stream settles it too: there is nothing more to wait for.
                let settled = read.is_empty()
                    || read[..len] != rest_of_preface[..len]
                    || len == rest_of_preface.len();

                if settled {
                    this.sniffing = None;
                } else {
                    sniffing.preface_matched += len;
                }

                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => {
                if sniffing.deadline.as_mut().poll(cx).is_ready() {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "the client did not start its request in time",
                    )));
                }

                Poll::Pending
            }
        }
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for ProtocolSniffDeadline<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.stream.is_write_vectored()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};

    const DEADLINE: Duration = Duration::from_millis(100);

    /// Longer than [`DEADLINE`], by enough that a deadline which did apply would have passed.
    const PAST_DEADLINE: Duration = Duration::from_millis(300);

    /// The server's end, with the deadline on it, and the client's end.
    fn connection() -> (ProtocolSniffDeadline<DuplexStream>, DuplexStream) {
        let (server, client) = tokio::io::duplex(1024);
        (ProtocolSniffDeadline::new(server, Some(DEADLINE)), client)
    }

    /// Reads into a buffer of 24 bytes at most, the way the auto builder does.
    async fn read(stream: &mut ProtocolSniffDeadline<DuplexStream>) -> io::Result<Vec<u8>> {
        let mut buf = [0u8; 24];
        let read = stream.read(&mut buf).await?;
        Ok(buf[..read].to_vec())
    }

    async fn assert_timed_out(stream: &mut ProtocolSniffDeadline<DuplexStream>) {
        let result = tokio::time::timeout(PAST_DEADLINE * 3, read(stream))
            .await
            .expect("the deadline did not apply");
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }

    async fn assert_still_waiting(stream: &mut ProtocolSniffDeadline<DuplexStream>) {
        let result = tokio::time::timeout(PAST_DEADLINE, read(stream)).await;
        assert!(result.is_err(), "the read did not wait: {result:?}");
    }

    #[tokio::test]
    async fn a_client_that_sends_nothing_times_out() {
        let (mut server, _client) = connection();
        assert_timed_out(&mut server).await;
    }

    #[tokio::test]
    async fn a_client_that_stops_inside_the_preface_times_out() {
        let (mut server, mut client) = connection();

        client.write_all(b"P").await.unwrap();
        assert_eq!(read(&mut server).await.unwrap(), b"P");

        client.write_all(b"RI * HTTP/2.0\r\n").await.unwrap();
        assert_eq!(read(&mut server).await.unwrap(), b"RI * HTTP/2.0\r\n");

        assert_timed_out(&mut server).await;
    }

    #[tokio::test]
    async fn the_first_byte_off_the_preface_lifts_the_deadline() {
        for first_bytes in [&b"G"[..], b"PO", b"PRI * HTTP/1.1"] {
            let (mut server, mut client) = connection();

            client.write_all(first_bytes).await.unwrap();
            assert_eq!(read(&mut server).await.unwrap(), first_bytes);

            assert_still_waiting(&mut server).await;
        }
    }

    #[tokio::test]
    async fn the_whole_preface_lifts_the_deadline() {
        let (mut server, mut client) = connection();

        // In two reads, and the second one carrying more than the rest of the preface.
        client.write_all(&H2_PREFACE[..10]).await.unwrap();
        assert_eq!(read(&mut server).await.unwrap(), &H2_PREFACE[..10]);

        let mut rest = H2_PREFACE[10..].to_vec();
        rest.extend_from_slice(b"frames");
        client.write_all(&rest).await.unwrap();

        let mut received = Vec::new();
        while received.len() < rest.len() {
            received.extend(read(&mut server).await.unwrap());
        }
        assert_eq!(received, rest);

        assert_still_waiting(&mut server).await;
    }

    #[tokio::test]
    async fn without_a_timeout_there_is_no_deadline() {
        let (server, _client) = tokio::io::duplex(1024);
        let mut server = ProtocolSniffDeadline::new(server, None);

        assert_still_waiting(&mut server).await;
    }

    #[tokio::test]
    async fn writes_pass_through() {
        let (mut server, mut client) = connection();

        server.write_all(b"response").await.unwrap();
        server.flush().await.unwrap();

        let mut buf = [0u8; 8];
        client.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"response");

        assert_eq!(
            server.is_write_vectored(),
            server.stream.is_write_vectored()
        );
    }
}
