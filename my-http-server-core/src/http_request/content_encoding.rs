use std::io::Read;

use hyper::body::Bytes;

use crate::HttpFailResult;

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
    pub fn decompress_if_needed(&self, body: Bytes) -> Result<Vec<u8>, HttpFailResult> {
        let body: Vec<_> = body.into();
        match self {
            ContentEncoding::None => Ok(body.into()),
            ContentEncoding::GZip => match decompress_gzip(body.as_slice()) {
                Some(result) => return Ok(result),
                None => {
                    return self.decompress_fall_back(body.as_slice());
                }
            },
            ContentEncoding::Br => match decompress_br(body.as_slice()) {
                Some(result) => return Ok(result),
                None => {
                    return self.decompress_fall_back(body.as_slice());
                }
            },
            ContentEncoding::Zstd => match decompress_zstd(body.as_slice()) {
                Some(result) => return Ok(result),
                None => {
                    return self.decompress_fall_back(body.as_slice());
                }
            },
            ContentEncoding::Deflate => match decompress_deflate(body.as_slice()) {
                Some(result) => return Ok(result),
                None => {
                    return self.decompress_fall_back(body.as_slice());
                }
            },
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
    fn decompress_fall_back(&self, body: &[u8]) -> Result<Vec<u8>, HttpFailResult> {
        if *self != ContentEncoding::GZip {
            if let Some(body) = decompress_gzip(body) {
                return Ok(body);
            }
        }

        if *self != ContentEncoding::Zstd {
            if let Some(body) = decompress_zstd(body) {
                return Ok(body);
            }
        }

        if *self != ContentEncoding::Deflate {
            if let Some(body) = decompress_zlib(body) {
                return Ok(body);
            }
        }

        if *self != ContentEncoding::Br {
            if let Some(body) = decompress_br(body) {
                return Ok(body);
            }
        }

        Err(HttpFailResult::as_fatal_error(format!(
            "Can not decompress body using {:?} method",
            self
        )))
    }
}

fn decompress_gzip(body: &[u8]) -> Option<Vec<u8>> {
    read_to_end(flate2::read::GzDecoder::new(body))
}

/// Drains a decoder, or gives up as soon as it says the bytes are not its own. `None` is not an
/// error yet: the caller still has the other codecs to try.
fn read_to_end(mut decompressor: impl Read) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut buffer = [0u8; 1024 * 8];

    loop {
        let read_amount = decompressor.read(&mut buffer);

        if read_amount.is_err() {
            return None;
        }

        let read_amount = read_amount.unwrap();

        if read_amount == 0 {
            return Some(result);
        }

        result.extend_from_slice(&buffer[..read_amount]);
    }
}

/// The HTTP `deflate` body: a zlib stream (RFC 1950) is what the spec means, so it is tried
/// first; a bare deflate stream (RFC 1951) is what a handful of clients send instead, and is the
/// fallback. Trying it the other way round would let the permissive one answer for both.
fn decompress_deflate(body: &[u8]) -> Option<Vec<u8>> {
    if let Some(result) = decompress_zlib(body) {
        return Some(result);
    }

    read_to_end(flate2::read::DeflateDecoder::new(body))
}

fn decompress_zlib(body: &[u8]) -> Option<Vec<u8>> {
    read_to_end(flate2::read::ZlibDecoder::new(body))
}

/// `ruzstd` is a pure-Rust zstd **decoder**, which is all a server does to a request body - and it
/// keeps the C toolchain the `zstd` crate needs out of the build.
fn decompress_zstd(body: &[u8]) -> Option<Vec<u8>> {
    let decompressor = ruzstd::decoding::StreamingDecoder::new(body).ok()?;

    read_to_end(decompressor)
}

fn decompress_br(body: &[u8]) -> Option<Vec<u8>> {
    use brotli_decompressor::Decompressor;

    read_to_end(Decompressor::new(body, 4096))
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
            .decompress_if_needed(BODY.to_vec().into())
            .unwrap();

        assert_eq!(body.as_slice(), BODY);
    }

    #[test]
    fn a_gzip_body_is_decompressed() {
        let body = ContentEncoding::GZip
            .decompress_if_needed(gzip(BODY).into())
            .unwrap();

        assert_eq!(body.as_slice(), BODY);
    }

    #[test]
    fn a_zstd_body_is_decompressed() {
        let compressed = zstd(BODY);
        assert_ne!(compressed.as_slice(), BODY);

        let body = ContentEncoding::Zstd
            .decompress_if_needed(compressed.into())
            .unwrap();

        assert_eq!(body.as_slice(), BODY);
    }

    /// Both formats that travel under the name `deflate`.
    #[test]
    fn a_deflate_body_is_decompressed_whichever_of_its_two_formats_it_is() {
        let zlib_body = ContentEncoding::Deflate
            .decompress_if_needed(zlib(BODY).into())
            .unwrap();
        assert_eq!(zlib_body.as_slice(), BODY);

        let bare_body = ContentEncoding::Deflate
            .decompress_if_needed(raw_deflate(BODY).into())
            .unwrap();
        assert_eq!(bare_body.as_slice(), BODY);
    }

    #[test]
    fn a_body_encoded_with_a_codec_other_than_the_announced_one_still_decodes() {
        let body = ContentEncoding::Zstd
            .decompress_if_needed(gzip(BODY).into())
            .unwrap();
        assert_eq!(body.as_slice(), BODY);

        let body = ContentEncoding::GZip
            .decompress_if_needed(zstd(BODY).into())
            .unwrap();
        assert_eq!(body.as_slice(), BODY);
    }
}
