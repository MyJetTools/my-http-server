use flate2::Compression;

/// How hard gzip works on a response: the CPU spent per response against the bytes saved on the
/// wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionLevel {
    /// Level 1: the least CPU - for large or hot responses.
    Fast,
    /// Level 6: what a dynamic response, compressed on every request, usually wants.
    #[default]
    Balanced,
    /// Level 9: the smallest output - for responses produced rarely and fetched often.
    Best,
    /// An exact level, `0..=9`. `0` only wraps the bytes in gzip framing without compressing
    /// them; anything above 9 is taken as 9.
    Level(u8),
}

impl CompressionLevel {
    pub(crate) fn to_gzip(self) -> Compression {
        match self {
            Self::Fast => Compression::fast(),
            Self::Balanced => Compression::new(6),
            Self::Best => Compression::best(),
            Self::Level(level) => Compression::new(level.min(9) as u32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_levels_map_to_their_numbers() {
        assert_eq!(CompressionLevel::Fast.to_gzip().level(), 1);
        assert_eq!(CompressionLevel::Balanced.to_gzip().level(), 6);
        assert_eq!(CompressionLevel::Best.to_gzip().level(), 9);
        assert_eq!(CompressionLevel::default(), CompressionLevel::Balanced);
    }

    #[test]
    fn an_exact_level_is_capped_at_9() {
        assert_eq!(CompressionLevel::Level(0).to_gzip().level(), 0);
        assert_eq!(CompressionLevel::Level(4).to_gzip().level(), 4);
        assert_eq!(CompressionLevel::Level(200).to_gzip().level(), 9);
    }
}
