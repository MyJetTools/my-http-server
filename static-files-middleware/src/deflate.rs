use std::io::{Read, Write};

use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};

pub const DEFLATE_MIN_SIZE: usize = 10 * 1024;
const DEFLATE_EFFICIENCY_NUM: usize = 80;
const DEFLATE_EFFICIENCY_DEN: usize = 100;

pub fn try_deflate(raw: &[u8]) -> Option<Vec<u8>> {
    if raw.len() <= DEFLATE_MIN_SIZE {
        return None;
    }

    let mut encoder = DeflateEncoder::new(Vec::with_capacity(raw.len()), Compression::default());
    encoder.write_all(raw).ok()?;
    let compressed = encoder.finish().ok()?;

    if compressed.len() * DEFLATE_EFFICIENCY_DEN <= raw.len() * DEFLATE_EFFICIENCY_NUM {
        Some(compressed)
    } else {
        None
    }
}

pub fn inflate(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_file_not_deflated() {
        let raw = vec![b'a'; 5 * 1024];
        assert!(try_deflate(&raw).is_none());
    }

    #[test]
    fn highly_compressible_is_deflated() {
        let raw = vec![b'a'; 50 * 1024];
        let compressed = try_deflate(&raw).expect("should compress");
        assert!(compressed.len() * 100 <= raw.len() * 80);
        let roundtrip = inflate(&compressed).unwrap();
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
        assert!(try_deflate(&raw).is_none());
    }

    #[test]
    fn exactly_threshold_not_deflated() {
        let raw = vec![b'a'; DEFLATE_MIN_SIZE];
        assert!(try_deflate(&raw).is_none());
    }
}
