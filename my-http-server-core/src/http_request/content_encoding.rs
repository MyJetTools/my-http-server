use std::io::Read;

use hyper::body::Bytes;

use crate::HttpFailResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentEncoding {
    None,
    GZip,
    Br,
}

impl ContentEncoding {
    pub fn new(header_value: Option<&str>) -> Result<Self, HttpFailResult> {
        let header_value = match header_value {
            Some(value) => value,
            None => return Ok(Self::None),
        };

        if header_value.eq_ignore_ascii_case("gzip") {
            return Ok(Self::GZip);
        }

        if header_value.eq_ignore_ascii_case("br") {
            return Ok(Self::Br);
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
        }
    }

    fn decompress_fall_back(&self, body: &[u8]) -> Result<Vec<u8>, HttpFailResult> {
        match self {
            ContentEncoding::None => {}
            ContentEncoding::GZip => {
                if let Some(body) = decompress_br(body) {
                    return Ok(body);
                }
            }
            ContentEncoding::Br => {
                if let Some(body) = decompress_gzip(body) {
                    return Ok(body);
                }
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

fn decompress_br(body: &[u8]) -> Option<Vec<u8>> {
    use brotli::Decompressor;

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
