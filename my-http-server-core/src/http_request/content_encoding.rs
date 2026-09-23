use std::io::Read;

use hyper::body::Bytes;

use crate::HttpFailResult;

/// How large a request body may grow once decompressed, unless
/// [`MyHttpServer::set_max_decompressed_body_size`](crate::MyHttpServer::set_max_decompressed_body_size)
/// says otherwise.
///
/// Without a limit a request body is a decompression bomb waiting to happen: 10 MB of gzipped
/// zeros inflate to about 10 GB, and zstd and brotli do better still. 64 MiB is far above any JSON
/// or form a client means to send, and far below what takes a server down.
pub const DEFAULT_MAX_DECOMPRESSED_BODY_SIZE: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentEncoding {
    None,
    GZip,
    Br,
    /// Decoding only - the server never answers with zstd.
    Zstd,
    /// `deflate` names two formats in the wild: the zlib stream the HTTP spec means (RFC 1950),
    /// and the bare deflate stream some clients send instead (RFC 1951). Both are read.
    Deflate,
}

impl ContentEncoding {
    pub fn new(header_value: Option<&str>) -> Result<Self, HttpFailResult> {
        let header_value = match header_value {
            Some(value) => value.trim(),
            None => return Ok(Self::None),
        };

        // "identity" is the explicit way of saying "not encoded" - it must not be an error.
        if header_value.is_empty() || header_value.eq_ignore_ascii_case("identity") {
            return Ok(Self::None);
        }

        if header_value.eq_ignore_ascii_case("gzip") || header_value.eq_ignore_ascii_case("x-gzip")
        {
            return Ok(Self::GZip);
        }

        if header_value.eq_ignore_ascii_case("br") {
            return Ok(Self::Br);
        }

        if header_value.eq_ignore_ascii_case("zstd") {
            return Ok(Self::Zstd);
        }

        if header_value.eq_ignore_ascii_case("deflate") {
            return Ok(Self::Deflate);
        }

        Err(HttpFailResult::as_validation_error(format!(
            "Unsupported content encoding: {}",
            header_value
        )))
    }

    /// Undoes the announced encoding. A body that announced none is returned as it is, whatever its
    /// size - `max_decompressed_size` bounds only what a decoder produces.
    pub fn decompress_if_needed(
        &self,
        body: Bytes,
        max_decompressed_size: usize,
    ) -> Result<Vec<u8>, HttpFailResult> {
        let body: Vec<_> = body.into();

        let result = match self {
            ContentEncoding::None => return Ok(body),
            ContentEncoding::GZip => decompress_gzip(body.as_slice(), max_decompressed_size),
            ContentEncoding::Br => decompress_br(body.as_slice(), max_decompressed_size),
            ContentEncoding::Zstd => decompress_zstd(body.as_slice(), max_decompressed_size),
            ContentEncoding::Deflate => decompress_deflate(body.as_slice(), max_decompressed_size),
        };

        match result.into_final(max_decompressed_size) {
            Some(result) => result,
            None => self.decompress_fall_back(body.as_slice(), max_decompressed_size),
        }
    }

    /// The announced codec did not decode the body - so the header is probably wrong. Try the
    /// other ones before giving up: a body that decodes is the body the client meant to send.
    ///
    /// Order matters, and it is the order of how sure a decoder can be that the bytes are its
    /// own: gzip and zstd start with a magic number, zlib has a header and a checksum, and brotli
    /// has neither - it will happily turn arbitrary bytes into arbitrary bytes, so it goes last.
    /// Bare deflate (RFC 1951) is not in the chain at all for the same reason, only more so: it
    /// is tried when the client actually announced `deflate`, never on a guess.
    ///
    /// A codec that ran into the limit ends the chain on the spot, see
    /// [`DecompressOutcome::LimitExceeded`].
    fn decompress_fall_back(
        &self,
        body: &[u8],
        max_decompressed_size: usize,
    ) -> Result<Vec<u8>, HttpFailResult> {
        if *self != ContentEncoding::GZip {
            if let Some(result) =
                decompress_gzip(body, max_decompressed_size).into_final(max_decompressed_size)
            {
                return result;
            }
        }

        if *self != ContentEncoding::Zstd {
            if let Some(result) =
                decompress_zstd(body, max_decompressed_size).into_final(max_decompressed_size)
            {
                return result;
            }
        }

        if *self != ContentEncoding::Deflate {
            if let Some(result) =
                decompress_zlib(body, max_decompressed_size).into_final(max_decompressed_size)
            {
                return result;
            }
        }

        if *self != ContentEncoding::Br {
            if let Some(result) =
                decompress_br(body, max_decompressed_size).into_final(max_decompressed_size)
            {
                return result;
            }
        }

        // The client's mistake, not ours: the body is simply not what the header says, nor any
        // other encoding we know - so 400, not 500.
        Err(HttpFailResult::as_validation_error(format!(
            "Can not decompress the request body: it does not decode as {}, the announced Content-Encoding, nor as any other supported encoding",
            self.as_header_value()
        )))
    }

    /// The name the header uses for this encoding - what a client recognises in an error.
    fn as_header_value(&self) -> &'static str {
        match self {
            ContentEncoding::None => "identity",
            ContentEncoding::GZip => "gzip",
            ContentEncoding::Br => "br",
            ContentEncoding::Zstd => "zstd",
            ContentEncoding::Deflate => "deflate",
        }
    }
}

/// What one decoder made of a body. Three answers, not two, because "the bytes are not mine" and
/// "the bytes are mine, and too much" call for opposite reactions from the caller.
#[derive(Debug)]
enum DecompressOutcome {
    /// The bytes were this codec's, and this is what they decode to - within the limit.
    Decompressed(Vec<u8>),
    /// The bytes are not this codec's. Not an error yet: the caller still has the other codecs
    /// to try.
    NotThisCodec,
    /// The bytes decode to more than the limit allows. This is **final**: it must not send the
    /// caller on to the next codec. A bomb is a perfectly valid stream of its own codec - the
    /// other codecs would only fail on it, and a codec in the chain that happens to accept it
    /// would inflate it up to the limit all over again.
    LimitExceeded,
}

impl DecompressOutcome {
    /// The answer to hand the client, or `None` when the next codec should have a go.
    fn into_final(self, max_decompressed_size: usize) -> Option<Result<Vec<u8>, HttpFailResult>> {
        match self {
            DecompressOutcome::Decompressed(body) => Some(Ok(body)),
            DecompressOutcome::NotThisCodec => None,
            DecompressOutcome::LimitExceeded => Some(Err(HttpFailResult::from((
                413u16,
                format!(
                    "Payload Too Large: the request body decompresses to more than {} bytes, the most this server accepts",
                    max_decompressed_size
                ),
            )))),
        }
    }
}

fn decompress_gzip(body: &[u8], max_size: usize) -> DecompressOutcome {
    read_to_end(flate2::read::GzDecoder::new(body), max_size)
}

/// How much [`read_to_end`] asks a decoder for at a time - and so how far past the limit a decoder
/// can get before it is stopped.
const READ_BUFFER_SIZE: usize = 1024 * 8;

/// Drains a decoder - unless it says the bytes are not its own, or they turn out to decode to
/// more than `max_size`.
///
/// The limit is checked before each chunk is kept, not after the whole body is inflated: a bomb
/// is stopped within one buffer of the limit, having cost no more than the limit in memory.
fn read_to_end(mut decompressor: impl Read, max_size: usize) -> DecompressOutcome {
    let mut result = Vec::new();
    let mut buffer = [0u8; READ_BUFFER_SIZE];

    loop {
        let read_amount = decompressor.read(&mut buffer);

        if read_amount.is_err() {
            return DecompressOutcome::NotThisCodec;
        }

        let read_amount = read_amount.unwrap();

        if read_amount == 0 {
            return DecompressOutcome::Decompressed(result);
        }

        if read_amount > max_size - result.len() {
            return DecompressOutcome::LimitExceeded;
        }

        if result.capacity() - result.len() < read_amount {
            // Grow by doubling, as `Vec` would - but never past the limit. Plain doubling could
            // reserve nearly twice the limit for a body that ends just past a power of two.
            let new_capacity = result
                .capacity()
                .saturating_mul(2)
                .clamp(result.len() + read_amount, max_size);
            result.reserve_exact(new_capacity - result.len());
        }

        result.extend_from_slice(&buffer[..read_amount]);
    }
}

/// The HTTP `deflate` body: a zlib stream (RFC 1950) is what the spec means, so it is tried
/// first; a bare deflate stream (RFC 1951) is what a handful of clients send instead, and is the
/// fallback. Trying it the other way round would let the permissive one answer for both.
///
/// Only a zlib stream that is *not* one sends us on to bare deflate: one that ran into the limit
/// is a zlib stream all right, and the answer stands.
fn decompress_deflate(body: &[u8], max_size: usize) -> DecompressOutcome {
    match decompress_zlib(body, max_size) {
        DecompressOutcome::NotThisCodec => {}
        outcome => return outcome,
    }

    read_to_end(flate2::read::DeflateDecoder::new(body), max_size)
}

fn decompress_zlib(body: &[u8], max_size: usize) -> DecompressOutcome {
    read_to_end(flate2::read::ZlibDecoder::new(body), max_size)
}

/// `ruzstd` is a pure-Rust zstd **decoder**, which is all a server does to a request body - and it
/// keeps the C toolchain the `zstd` crate needs out of the build.
fn decompress_zstd(body: &[u8], max_size: usize) -> DecompressOutcome {
    let Ok(decompressor) = ruzstd::decoding::StreamingDecoder::new(body) else {
        return DecompressOutcome::NotThisCodec;
    };

    read_to_end(decompressor, max_size)
}

/// The first seven bits of a "Large Window Brotli" stream - the WBITS escape `0010001`, read from
/// the least significant bit - which RFC 7932 leaves invalid.
const BROTLI_LARGE_WINDOW_MARKER: u8 = 0x11;

/// `brotli_decompressor::Decompressor` accepts "Large Window Brotli" - a non-standard extension,
/// not part of the `br` content coding (RFC 7932) - and sizes its ring buffer from the stream's
/// own header before it produces a single byte: up to 1 GiB, from a 6-byte body. The output limit
/// can not catch that, it comes too late. A strict RFC 7932 decoder rejects the header, and so do
/// we, before decoding: it is simply not a `br` body. Standard brotli tops out at a 16 MiB window.
fn decompress_br(body: &[u8], max_size: usize) -> DecompressOutcome {
    use brotli_decompressor::Decompressor;

    if body
        .first()
        .is_some_and(|first| first & 0x7F == BROTLI_LARGE_WINDOW_MARKER)
    {
        return DecompressOutcome::NotThisCodec;
    }

    read_to_end(Decompressor::new(body, 4096), max_size)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn gzip(raw: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(raw).unwrap();
        encoder.finish().unwrap()
    }

    fn zstd(raw: &[u8]) -> Vec<u8> {
        ruzstd::encoding::compress_to_vec(raw, ruzstd::encoding::CompressionLevel::Fastest)
    }

    /// What the HTTP spec means by `deflate`: a zlib stream.
    fn zlib(raw: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(raw).unwrap();
        encoder.finish().unwrap()
    }

    /// What some clients send under that same name: a bare deflate stream.
    fn raw_deflate(raw: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(raw).unwrap();
        encoder.finish().unwrap()
    }

    const BODY: &[u8] = br#"{"email":"a@b.com","name":"John Doe, who compresses well"}"#;

    // There is no brotli *encoder* among the dependencies - the server never answers with br - so
    // the brotli bodies are fixtures, made with the `brotli` 8.0.2 crate and checked to round-trip.

    /// `BODY`, standard brotli (quality 5, lgwin 22).
    const BR_BODY: &[u8] = &[
        0x1b, 0x39, 0x00, 0x00, 0x44, 0xe7, 0x96, 0xea, 0xf7, 0x4c, 0x48, 0x24, 0x83, 0xc9, 0xd2,
        0xd0, 0xb1, 0x51, 0x90, 0x60, 0x30, 0x38, 0x61, 0x22, 0x27, 0xe0, 0x2d, 0x0a, 0xf4, 0x80,
        0xf8, 0xd4, 0xba, 0xd3, 0xc7, 0x28, 0x53, 0x5a, 0x9b, 0x27, 0xc3, 0x43, 0xe1, 0xb6, 0x74,
        0x2a, 0x47, 0xc5, 0x46, 0x2f, 0x17, 0xdc, 0x28, 0xac, 0x4a, 0x41, 0x7e, 0x80, 0x81, 0x1f,
    ];

    /// The same `BODY`, as "Large Window Brotli" (lgwin 30) - note the `0x11` it starts with.
    const BR_BODY_LARGE_WINDOW: &[u8] = &[
        0x11, 0x5e, 0xe4, 0x00, 0x00, 0x10, 0x9d, 0x5b, 0xaa, 0xdf, 0x33, 0x21, 0x91, 0x0c, 0x26,
        0x4b, 0x43, 0xc7, 0x46, 0x41, 0x82, 0xc1, 0xe0, 0x84, 0x89, 0x9c, 0x80, 0xb7, 0x28, 0xd0,
        0x03, 0xe2, 0x53, 0xa3, 0xe3, 0xc4, 0xc7, 0x28, 0x53, 0x5a, 0x9b, 0x27, 0xc3, 0x43, 0xe1,
        0xb6, 0x74, 0x2a, 0x47, 0xc5, 0x46, 0x2f, 0x17, 0xdc, 0x28, 0xac, 0x4a, 0x41, 0x7e, 0x80,
        0x81, 0x1f,
    ];

    /// 64 KiB of `a` - exactly [`SMALL_LIMIT`] bytes.
    const BR_SMALL_LIMIT_OF_A: &[u8] = &[
        0x1b, 0xff, 0xff, 0x00, 0x24, 0xc2, 0xe2, 0xb1, 0x40, 0x72, 0xef, 0x01, 0x00,
    ];

    /// 8 MiB of zeros - [`BOMB_SIZE`] - in 14 bytes.
    const BR_BOMB: &[u8] = &[
        0xcb, 0xff, 0xff, 0x3f, 0x00, 0x24, 0x00, 0xe2, 0xb1, 0x40, 0x72, 0xef, 0xff, 0x06,
    ];

    #[test]
    fn an_absent_or_identity_header_means_no_encoding() {
        assert_eq!(ContentEncoding::new(None).unwrap(), ContentEncoding::None);
        assert_eq!(
            ContentEncoding::new(Some("")).unwrap(),
            ContentEncoding::None
        );
        assert_eq!(
            ContentEncoding::new(Some(" identity ")).unwrap(),
            ContentEncoding::None
        );
    }

    #[test]
    fn the_header_is_matched_case_insensitively() {
        assert_eq!(
            ContentEncoding::new(Some("GZip")).unwrap(),
            ContentEncoding::GZip
        );
        assert_eq!(
            ContentEncoding::new(Some("x-gzip")).unwrap(),
            ContentEncoding::GZip
        );
        assert_eq!(ContentEncoding::new(Some("BR")).unwrap(), ContentEncoding::Br);
        assert_eq!(
            ContentEncoding::new(Some("zstd")).unwrap(),
            ContentEncoding::Zstd
        );
        assert_eq!(
            ContentEncoding::new(Some("Deflate")).unwrap(),
            ContentEncoding::Deflate
        );
    }

    #[test]
    fn an_encoding_we_can_not_undo_is_rejected() {
        assert!(ContentEncoding::new(Some("compress")).is_err());
        // A list of codecs is not supported either - we decode exactly one layer.
        assert!(ContentEncoding::new(Some("gzip, br")).is_err());
    }

    #[test]
    fn an_unencoded_body_is_passed_through_untouched() {
        let body = ContentEncoding::None
            .decompress_if_needed(BODY.to_vec().into(), DEFAULT_MAX_DECOMPRESSED_BODY_SIZE)
            .unwrap();

        assert_eq!(body.as_slice(), BODY);
    }

    #[test]
    fn a_gzip_body_is_decompressed() {
        let body = ContentEncoding::GZip
            .decompress_if_needed(gzip(BODY).into(), DEFAULT_MAX_DECOMPRESSED_BODY_SIZE)
            .unwrap();

        assert_eq!(body.as_slice(), BODY);
    }

    #[test]
    fn a_zstd_body_is_decompressed() {
        let compressed = zstd(BODY);
        assert_ne!(compressed.as_slice(), BODY);

        let body = ContentEncoding::Zstd
            .decompress_if_needed(compressed.into(), DEFAULT_MAX_DECOMPRESSED_BODY_SIZE)
            .unwrap();

        assert_eq!(body.as_slice(), BODY);
    }

    #[test]
    fn a_br_body_is_decompressed() {
        let body = ContentEncoding::Br
            .decompress_if_needed(BR_BODY.to_vec().into(), DEFAULT_MAX_DECOMPRESSED_BODY_SIZE)
            .unwrap();

        assert_eq!(body.as_slice(), BODY);
    }

    /// Both formats that travel under the name `deflate`.
    #[test]
    fn a_deflate_body_is_decompressed_whichever_of_its_two_formats_it_is() {
        let zlib_body = ContentEncoding::Deflate
            .decompress_if_needed(zlib(BODY).into(), DEFAULT_MAX_DECOMPRESSED_BODY_SIZE)
            .unwrap();
        assert_eq!(zlib_body.as_slice(), BODY);

        let bare_body = ContentEncoding::Deflate
            .decompress_if_needed(raw_deflate(BODY).into(), DEFAULT_MAX_DECOMPRESSED_BODY_SIZE)
            .unwrap();
        assert_eq!(bare_body.as_slice(), BODY);
    }

    #[test]
    fn a_body_encoded_with_a_codec_other_than_the_announced_one_still_decodes() {
        let body = ContentEncoding::Zstd
            .decompress_if_needed(gzip(BODY).into(), DEFAULT_MAX_DECOMPRESSED_BODY_SIZE)
            .unwrap();
        assert_eq!(body.as_slice(), BODY);

        let body = ContentEncoding::GZip
            .decompress_if_needed(zstd(BODY).into(), DEFAULT_MAX_DECOMPRESSED_BODY_SIZE)
            .unwrap();
        assert_eq!(body.as_slice(), BODY);
    }

    /// The status and the text a rejected body is answered with.
    fn failure(result: Result<Vec<u8>, HttpFailResult>) -> (u16, String) {
        let err = result.expect_err("the body was expected to be rejected");

        let text = match &err.output {
            crate::HttpOutput::Content { content, .. } => {
                String::from_utf8_lossy(content).to_string()
            }
            _ => String::new(),
        };

        (err.output.get_status_code(), text)
    }

    /// Passes a decoder's output through, counting it - to see where [`read_to_end`] stopped it.
    struct CountingReader<R> {
        inner: R,
        produced: usize,
    }

    impl<R: Read> Read for CountingReader<R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let read_amount = self.inner.read(buf)?;
            self.produced += read_amount;
            Ok(read_amount)
        }
    }

    /// 8 MiB of zeros - a few kilobytes on the wire.
    const BOMB_SIZE: usize = 8 * 1024 * 1024;
    const SMALL_LIMIT: usize = 64 * 1024;

    fn gzip_bomb() -> Vec<u8> {
        gzip(&vec![0u8; BOMB_SIZE])
    }

    /// Bytes that are no encoding at all, sent under a header that says they are. The client got it
    /// wrong, so 400 - and the text says that nothing decoded it, not merely the announced codec.
    #[test]
    fn a_body_that_does_not_decode_at_all_is_a_400() {
        let (status, text) = failure(ContentEncoding::GZip.decompress_if_needed(
            b"this is not a compressed body".to_vec().into(),
            DEFAULT_MAX_DECOMPRESSED_BODY_SIZE,
        ));

        assert_eq!(status, 400, "{}", text);
        assert!(text.contains("gzip"), "{}", text);
        assert!(text.contains("nor as any other"), "{}", text);
    }

    /// The same bomb, in every codec we decode.
    fn bombs() -> [(ContentEncoding, Vec<u8>); 4] {
        let zeros = vec![0u8; BOMB_SIZE];

        [
            (ContentEncoding::GZip, gzip(&zeros)),
            (ContentEncoding::Zstd, zstd(&zeros)),
            (ContentEncoding::Deflate, zlib(&zeros)),
            (ContentEncoding::Br, BR_BOMB.to_vec()),
        ]
    }

    #[test]
    fn a_bomb_is_rejected_with_a_413_naming_the_limit_whichever_codec_it_comes_in() {
        for (encoding, bomb) in bombs() {
            let (status, text) = failure(encoding.decompress_if_needed(bomb.into(), SMALL_LIMIT));

            assert_eq!(status, 413, "{:?}: {}", encoding, text);
            assert!(text.contains(&SMALL_LIMIT.to_string()), "{}", text);
        }
    }

    /// The point of checking the limit chunk by chunk: the decoder is stopped within one buffer of
    /// it, not after it has produced all 8 MiB.
    #[test]
    fn a_bomb_is_stopped_at_the_limit_not_after_inflating_it_whole() {
        for (encoding, bomb) in bombs() {
            let decoder: Box<dyn Read> = match encoding {
                ContentEncoding::GZip => Box::new(flate2::read::GzDecoder::new(bomb.as_slice())),
                ContentEncoding::Zstd => Box::new(
                    ruzstd::decoding::StreamingDecoder::new(bomb.as_slice()).unwrap(),
                ),
                ContentEncoding::Deflate => {
                    Box::new(flate2::read::ZlibDecoder::new(bomb.as_slice()))
                }
                ContentEncoding::Br => Box::new(brotli_decompressor::Decompressor::new(
                    bomb.as_slice(),
                    4096,
                )),
                ContentEncoding::None => unreachable!(),
            };

            let mut decoder = CountingReader {
                inner: decoder,
                produced: 0,
            };

            let outcome = read_to_end(&mut decoder, SMALL_LIMIT);

            assert!(
                matches!(outcome, DecompressOutcome::LimitExceeded),
                "{:?}",
                encoding
            );
            assert!(
                decoder.produced <= SMALL_LIMIT + READ_BUFFER_SIZE,
                "{:?}: the decoder produced {} bytes for a limit of {}",
                encoding,
                decoder.produced,
                SMALL_LIMIT
            );
        }
    }

    /// The three answers a decoder gives, and which of them let the next codec have a go.
    #[test]
    fn a_decoder_tells_not_mine_apart_from_too_large() {
        match decompress_gzip(&gzip(BODY), SMALL_LIMIT) {
            DecompressOutcome::Decompressed(body) => assert_eq!(body.as_slice(), BODY),
            outcome => panic!("expected Decompressed, got {:?}", outcome),
        }

        assert!(matches!(
            decompress_gzip(&zstd(BODY), SMALL_LIMIT),
            DecompressOutcome::NotThisCodec
        ));

        assert!(matches!(
            decompress_gzip(&gzip_bomb(), SMALL_LIMIT),
            DecompressOutcome::LimitExceeded
        ));

        // Only "not mine" sends the caller on to the next codec.
        assert!(DecompressOutcome::NotThisCodec
            .into_final(SMALL_LIMIT)
            .is_none());
        assert!(matches!(
            DecompressOutcome::LimitExceeded.into_final(SMALL_LIMIT),
            Some(Err(_))
        ));
    }

    /// A bomb announced under the wrong name is found by the fallback - and ends the chain there.
    /// Had the limit sent it on to the remaining codecs, none of them would decode gzip and the
    /// answer would be the 400 of "nothing decoded it".
    #[test]
    fn a_bomb_that_hits_the_limit_does_not_go_on_to_the_other_codecs() {
        for announced in [
            ContentEncoding::GZip,
            ContentEncoding::Zstd,
            ContentEncoding::Deflate,
            ContentEncoding::Br,
        ] {
            let (status, text) =
                failure(announced.decompress_if_needed(gzip_bomb().into(), SMALL_LIMIT));

            assert_eq!(status, 413, "announced {:?}: {}", announced, text);
        }
    }

    /// Same inside `deflate`: a zlib stream that hits the limit is not retried as bare deflate.
    #[test]
    fn a_zlib_bomb_is_not_retried_as_bare_deflate() {
        assert!(matches!(
            decompress_deflate(&zlib(&vec![0u8; BOMB_SIZE]), SMALL_LIMIT),
            DecompressOutcome::LimitExceeded
        ));
    }

    /// The limit is inclusive: a body that decodes to exactly the limit is taken, one byte more is
    /// not - whichever codec it comes in.
    #[test]
    fn a_body_exactly_at_the_limit_passes_and_one_byte_more_does_not() {
        let raw = vec![b'a'; SMALL_LIMIT];

        let encoded = [
            (ContentEncoding::GZip, gzip(&raw)),
            (ContentEncoding::Zstd, zstd(&raw)),
            (ContentEncoding::Deflate, zlib(&raw)),
            (ContentEncoding::Deflate, raw_deflate(&raw)),
            (ContentEncoding::Br, BR_SMALL_LIMIT_OF_A.to_vec()),
        ];

        for (encoding, body) in encoded {
            let decoded = encoding
                .decompress_if_needed(body.clone().into(), SMALL_LIMIT)
                .unwrap();
            assert_eq!(decoded, raw, "{:?}", encoding);

            let (status, text) =
                failure(encoding.decompress_if_needed(body.into(), SMALL_LIMIT - 1));
            assert_eq!(status, 413, "{:?}: {}", encoding, text);
        }
    }

    /// The limit bounds memory, not just length: the buffer does not double past it.
    #[test]
    fn the_decoded_body_never_reserves_more_than_the_limit() {
        // Not a power of two, and not a multiple of the read buffer - where plain `Vec` doubling
        // would overshoot the most.
        const LIMIT: usize = 100_000;

        match decompress_gzip(&gzip(&vec![b'a'; LIMIT]), LIMIT) {
            DecompressOutcome::Decompressed(body) => {
                assert_eq!(body.len(), LIMIT);
                assert!(body.capacity() <= LIMIT, "capacity {}", body.capacity());
            }
            outcome => panic!("expected Decompressed, got {:?}", outcome),
        }
    }

    /// The limit is about what a decoder produces. A body that announced no encoding is not
    /// decoded, so it is not held to it.
    #[test]
    fn an_unencoded_body_is_not_held_to_the_limit() {
        let raw = vec![b'a'; SMALL_LIMIT * 2];

        let body = ContentEncoding::None
            .decompress_if_needed(raw.clone().into(), SMALL_LIMIT)
            .unwrap();

        assert_eq!(body, raw);
    }

    /// The same body decodes as standard brotli and is refused as "Large Window Brotli": it is the
    /// header that is turned away, before the decoder could size a window from it.
    #[test]
    fn a_large_window_brotli_stream_is_refused_before_decoding() {
        assert_eq!(BR_BODY_LARGE_WINDOW[0] & 0x7F, BROTLI_LARGE_WINDOW_MARKER);

        assert!(matches!(
            decompress_br(BR_BODY_LARGE_WINDOW, DEFAULT_MAX_DECOMPRESSED_BODY_SIZE),
            DecompressOutcome::NotThisCodec
        ));

        // Not `br` - and not anything else either, so the client gets the 400 of a body that
        // does not decode.
        let (status, text) = failure(ContentEncoding::Br.decompress_if_needed(
            BR_BODY_LARGE_WINDOW.to_vec().into(),
            DEFAULT_MAX_DECOMPRESSED_BODY_SIZE,
        ));
        assert_eq!(status, 400, "{}", text);
    }
}
