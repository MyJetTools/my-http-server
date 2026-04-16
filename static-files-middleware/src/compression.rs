use std::io::{Read, Write};

use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression as DeflateCompression};

pub const COMPRESSION_MIN_SIZE: usize = 10 * 1024;
const EFFICIENCY_NUM: usize = 80;
const EFFICIENCY_DEN: usize = 100;
const ZSTD_LEVEL: i32 = 11;

pub fn try_zstd(raw: &[u8]) -> Option<Vec<u8>> {
    if raw.len() <= COMPRESSION_MIN_SIZE {
        return None;
    }

    let compressed = zstd::encode_all(raw, ZSTD_LEVEL).ok()?;

    if compressed.len() * EFFICIENCY_DEN <= raw.len() * EFFICIENCY_NUM {
        Some(compressed)
    } else {
        None
    }
}

pub fn zstd_decompress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    zstd::decode_all(data)
}

pub fn deflate_compress(raw: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder =
        DeflateEncoder::new(Vec::with_capacity(raw.len()), DeflateCompression::default());
    encoder.write_all(raw)?;
    encoder.finish()
}

#[allow(dead_code)]
pub fn deflate_decompress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_file_not_zstd() {
        let raw = vec![b'a'; 5 * 1024];
        assert!(try_zstd(&raw).is_none());
    }

    #[test]
    fn highly_compressible_is_zstd() {
        let raw = vec![b'a'; 50 * 1024];
        let compressed = try_zstd(&raw).expect("should compress");
        assert!(compressed.len() * 100 <= raw.len() * 80);
        let roundtrip = zstd_decompress(&compressed).unwrap();
        assert_eq!(roundtrip, raw);
    }

    #[test]
    fn incompressible_is_rejected() {
        let mut raw = Vec::with_capacity(50 * 1024);
        let mut seed: u32 = 0x9E37_79B1;
        for _ in 0..50 * 1024 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            raw.push((seed >> 24) as u8);
        }
        assert!(try_zstd(&raw).is_none());
    }

    #[test]
    fn exactly_threshold_not_compressed() {
        let raw = vec![b'a'; COMPRESSION_MIN_SIZE];
        assert!(try_zstd(&raw).is_none());
    }

    #[test]
    fn deflate_roundtrips() {
        let raw = vec![b'x'; 20 * 1024];
        let compressed = deflate_compress(&raw).unwrap();
        let roundtrip = deflate_decompress(&compressed).unwrap();
        assert_eq!(roundtrip, raw);
    }
}
