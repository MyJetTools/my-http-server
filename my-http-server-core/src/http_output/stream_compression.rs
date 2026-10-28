//! Compressing a streamed response on the fly - what
//! [`with_compression`](crate::HttpOutputAsStream::with_compression) runs under the hood.

use std::io::Write;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use flate2::write::GzEncoder;
use futures::{ready, Stream, StreamExt};
use hyper::body::Frame;
use hyper::HeaderMap;

use crate::CompressionLevel;

/// Whether the client takes a gzip response, read the way RFC 9110 §12.5.3 has it: a listed
/// `gzip` (or its alias `x-gzip`) decides by its weight, otherwise `*` does, and `q=0` means
/// "not acceptable". A weight that does not parse counts as `q=0` - sending the stream as it is
/// is always understood, compressing it for a client that may not take it is not.
pub(crate) fn accepts_gzip<'s>(accept_encoding: impl Iterator<Item = &'s str>) -> bool {
    let mut gzip = None;
    let mut any = None;

    for coding in accept_encoding.flat_map(|value| value.split(',')) {
        let mut parts = coding.split(';');
        let name = parts.next().unwrap_or_default().trim();

        let acceptable = parts
            .filter_map(|param| param.split_once('='))
            .find(|(key, _)| key.trim().eq_ignore_ascii_case("q"))
            .map_or(true, |(_, weight)| {
                weight.trim().parse::<f32>().map_or(false, |q| q > 0.0)
            });

        if name.eq_ignore_ascii_case("gzip") || name.eq_ignore_ascii_case("x-gzip") {
            gzip = Some(gzip.unwrap_or(false) || acceptable);
        } else if name == "*" {
            any = Some(any.unwrap_or(false) || acceptable);
        }
    }

    gzip.or(any).unwrap_or(false)
}

/// Gzips a stream of body frames as they pass through.
///
/// Every data frame is compressed and *sync-flushed*, so each one leaves as soon as it is
/// produced and the client can decode everything it has received so far - a slow producer is
/// still a stream, not a response held back until the end. The gzip footer (CRC32 and length)
/// goes out when the inner stream ends, which is when the producer is dropped: the stream is the
/// one that sees the end, so the footer can not be lost the way a send from `Drop` could.
///
/// Trailers, which have to come last, are held back until the footer is out. An error ends the
/// stream as it is - the response is broken anyway, and a footer would make it look complete.
pub(crate) struct GzipFrames<S> {
    frames: S,
    encoder: GzEncoder<Vec<u8>>,
    trailers: Option<HeaderMap>,
    finished: bool,
}

impl<S> GzipFrames<S> {
    pub fn new(frames: S, level: CompressionLevel) -> Self {
        Self {
            frames,
            encoder: GzEncoder::new(Vec::new(), level.to_gzip()),
            trailers: None,
            finished: false,
        }
    }

    fn compress(&mut self, data: &[u8]) -> Result<Frame<Bytes>, String> {
        self.encoder
            .write_all(data)
            .and_then(|_| self.encoder.flush())
            .map_err(|err| format!("Can not gzip the response stream: {}", err))?;

        Ok(self.take_output())
    }

    fn finish(&mut self) -> Result<Frame<Bytes>, String> {
        self.finished = true;

        self.encoder
            .try_finish()
            .map_err(|err| format!("Can not gzip the response stream: {}", err))?;

        Ok(self.take_output())
    }

    /// The encoder only ever appends to its output, so what it has written so far can be taken.
    fn take_output(&mut self) -> Frame<Bytes> {
        Frame::data(std::mem::take(self.encoder.get_mut()).into())
    }
}

impl<S, E> Stream for GzipFrames<S>
where
    S: Stream<Item = Result<Frame<Bytes>, E>> + Unpin,
    E: ToString,
{
    type Item = Result<Frame<Bytes>, String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.finished {
            return Poll::Ready(
                this.trailers
                    .take()
                    .map(|trailers| Ok(Frame::trailers(trailers))),
            );
        }

        loop {
            let frame = match ready!(this.frames.poll_next_unpin(cx)) {
                Some(Ok(frame)) => frame,
                Some(Err(err)) => {
                    this.finished = true;
                    return Poll::Ready(Some(Err(err.to_string())));
                }
                None => return Poll::Ready(Some(this.finish())),
            };

            match frame.into_data() {
                // A sync flush of nothing still costs a few bytes on the wire.
                Ok(data) if data.is_empty() => continue,
                Ok(data) => return Poll::Ready(Some(this.compress(&data))),
                Err(frame) => {
                    this.trailers = frame.into_trailers().ok();
                    return Poll::Ready(Some(this.finish()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    fn data(bytes: &[u8]) -> Result<Frame<Bytes>, String> {
        Ok(Frame::data(Bytes::copy_from_slice(bytes)))
    }

    fn run(frames: Vec<Result<Frame<Bytes>, String>>) -> Vec<Result<Frame<Bytes>, String>> {
        run_at(frames, CompressionLevel::Balanced)
    }

    fn run_at(
        frames: Vec<Result<Frame<Bytes>, String>>,
        level: CompressionLevel,
    ) -> Vec<Result<Frame<Bytes>, String>> {
        let gzip = GzipFrames::new(futures::stream::iter(frames), level);
        futures::executor::block_on(gzip.collect())
    }

    fn data_of(frames: &[Result<Frame<Bytes>, String>]) -> Vec<Vec<u8>> {
        frames
            .iter()
            .filter_map(|frame| frame.as_ref().ok()?.data_ref().map(|d| d.to_vec()))
            .collect()
    }

    /// What a client decodes from `gzip` - as much as there is, even if it is cut short.
    fn gunzip(gzip: &[u8]) -> (Vec<u8>, std::io::Result<usize>) {
        let mut out = Vec::new();
        let result = flate2::read::GzDecoder::new(gzip).read_to_end(&mut out);
        (out, result)
    }

    #[test]
    fn accepts_gzip_by_name_and_alias() {
        assert!(accepts_gzip(["gzip"].into_iter()));
        assert!(accepts_gzip(["GZip"].into_iter()));
        assert!(accepts_gzip(["x-gzip"].into_iter()));
        assert!(accepts_gzip(["br, gzip, deflate"].into_iter()));
        assert!(accepts_gzip(["br", "gzip"].into_iter()));
    }

    #[test]
    fn does_not_accept_gzip_when_not_listed() {
        assert!(!accepts_gzip([].into_iter()));
        assert!(!accepts_gzip([""].into_iter()));
        assert!(!accepts_gzip(["identity"].into_iter()));
        assert!(!accepts_gzip(["br, deflate, zstd"].into_iter()));
        assert!(!accepts_gzip(["gzip2"].into_iter()));
    }

    #[test]
    fn a_zero_weight_refuses_gzip() {
        assert!(!accepts_gzip(["gzip;q=0"].into_iter()));
        assert!(!accepts_gzip(["gzip; Q=0.000"].into_iter()));
        assert!(!accepts_gzip(["br, gzip ; q=0 , deflate"].into_iter()));
        assert!(accepts_gzip(["gzip;q=0.001"].into_iter()));
        assert!(accepts_gzip(["gzip;q=1.0, deflate;q=0.5"].into_iter()));
    }

    #[test]
    fn an_unparsable_weight_refuses_gzip() {
        assert!(!accepts_gzip(["gzip;q=high"].into_iter()));
    }

    #[test]
    fn the_wildcard_decides_only_for_an_unlisted_gzip() {
        assert!(accepts_gzip(["*"].into_iter()));
        assert!(!accepts_gzip(["*;q=0"].into_iter()));
        assert!(!accepts_gzip(["*, gzip;q=0"].into_iter()));
        assert!(accepts_gzip(["*;q=0, gzip"].into_iter()));
    }

    #[test]
    fn every_frame_is_decodable_as_soon_as_it_is_out() {
        let parts: [&[u8]; 3] = [b"first-part;", b"second-part;", b"third-part"];

        let frames = run(parts.iter().map(|part| data(part)).collect());
        let frames = data_of(&frames);

        // One frame per chunk produced, plus the footer.
        assert_eq!(frames.len(), parts.len() + 1);

        for received in 1..=parts.len() {
            let (decoded, _) = gunzip(&frames[..received].concat());
            assert_eq!(decoded, parts[..received].concat());
        }

        let (decoded, result) = gunzip(&frames.concat());
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(decoded, parts.concat());
    }

    #[test]
    fn without_the_footer_the_stream_does_not_decode_in_full() {
        let frames = run(vec![data(b"some content")]);
        let frames = data_of(&frames);

        let (_, result) = gunzip(&frames[..frames.len() - 1].concat());
        assert!(result.is_err());
    }

    /// Level 0 only frames the bytes: what comes out is larger than what went in, and still
    /// decodes - so the level the stream was given is the one the encoder runs at.
    #[test]
    fn the_level_is_applied() {
        let chunk = vec![b'a'; 4 * 1024];

        let stored = data_of(&run_at(vec![data(&chunk)], CompressionLevel::Level(0))).concat();
        let compressed = data_of(&run_at(vec![data(&chunk)], CompressionLevel::Fast)).concat();

        assert!(stored.len() > chunk.len(), "stored {}", stored.len());
        assert!(
            compressed.len() < chunk.len(),
            "compressed {}",
            compressed.len()
        );

        for gzip in [stored, compressed] {
            let (decoded, result) = gunzip(&gzip);
            assert!(result.is_ok(), "{:?}", result);
            assert_eq!(decoded, chunk);
        }
    }

    #[test]
    fn an_empty_stream_is_an_empty_gzip() {
        let frames = data_of(&run(vec![]));

        assert_eq!(frames.len(), 1);
        let (decoded, result) = gunzip(&frames[0]);
        assert!(result.is_ok(), "{:?}", result);
        assert!(decoded.is_empty());
    }

    #[test]
    fn empty_data_frames_are_skipped() {
        let frames = data_of(&run(vec![data(b""), data(b"abc"), data(b"")]));

        assert_eq!(frames.len(), 2);
        assert_eq!(gunzip(&frames.concat()).0, b"abc");
    }

    #[test]
    fn an_error_ends_the_stream_without_a_footer() {
        let frames = run(vec![
            data(b"abc"),
            Err("connection lost".to_string()),
            data(b"never sent"),
        ]);

        assert_eq!(frames.len(), 2);
        assert!(frames[0].is_ok());
        assert_eq!(frames[1].as_ref().unwrap_err(), "connection lost");
    }

    #[test]
    fn trailers_come_after_the_footer() {
        let mut trailers = HeaderMap::new();
        trailers.insert("x-checksum", "42".parse().unwrap());

        let frames = run(vec![
            data(b"abc"),
            Ok(Frame::trailers(trailers)),
            data(b"never sent"),
        ]);

        assert_eq!(frames.len(), 3);
        let last = frames[2].as_ref().unwrap().trailers_ref().unwrap();
        assert_eq!(last.get("x-checksum").unwrap(), "42");

        let (decoded, result) = gunzip(&data_of(&frames).concat());
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(decoded, b"abc");
    }
}
