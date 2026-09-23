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
        }
    }

    /// The announced codec did not decode the body - so the header is probably wrong. Try the
    /// other ones before giving up: a body that decodes is the body the client meant to send.
    ///
    /// Order matters. gzip and zstd both start with a magic number, so they recognise their own
    /// input and reject everything else; brotli has no header at all and will happily turn
    /// arbitrary bytes into arbitrary bytes, so it goes last - otherwise it would answer for a
    /// body that one of the others would have decoded properly.
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
    let mut decompressor = flate2::read::GzDecoder::new(body);

    let mut result = Vec::new();
    let mut buffer = [0u8; 1024 * 4];

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

/// `ruzstd` is a pure-Rust zstd **decoder**, which is all a server does to a request body - and it
/// keeps the C toolchain the `zstd` crate needs out of the build.
fn decompress_zstd(body: &[u8]) -> Option<Vec<u8>> {
    let mut decompressor = match ruzstd::decoding::StreamingDecoder::new(body) {
        Ok(decompressor) => decompressor,
        Err(_) => return None,
    };

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

fn decompress_br(body: &[u8]) -> Option<Vec<u8>> {
    use brotli_decompressor::Decompressor;

    let mut decompressor = Decompressor::new(body, 4096);

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
    }

    #[test]
    fn an_encoding_we_can_not_undo_is_rejected() {
        assert!(ContentEncoding::new(Some("deflate")).is_err());
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
