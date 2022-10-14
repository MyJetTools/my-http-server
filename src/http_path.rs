#[derive(Debug, Clone)]
pub struct HttpPath {
    pub path: Vec<u8>,
    segments: Vec<usize>,
}

impl HttpPath {
    pub fn new(path: &str) -> Self {
        let mut segments = Vec::new();

        let path = path.as_bytes().to_vec();

        if path.len() == 0 {
            panic!("Invalid http path. Path at least should contain one character '/'");
        }

        if path[0] != b'/' {
            panic!("Invalid http path. Path should start with '/'");
        }

        let mut last: u8 = 0;

        for pos in 0..path.len() {
            last = path[pos];
            if last == b'/' {
                segments.push(pos);
            }
        }

        if last != b'/' {
            segments.push(path.len());
        }

        Self { path, segments }
    }

    pub fn is_root(&self) -> bool {
        self.path.len() == 1
    }

    pub fn segments_amount(&self) -> usize {
        self.segments.len() - 1
    }

    pub fn get_segment_value(&self, index: usize) -> Option<&str> {
        let pos = self.segments.get(index + 1)?;
        let pos_prev = self.segments.get(index)?;

        let result = &self.path[*pos_prev + 1..*pos];
        Some(std::str::from_utf8(result).unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_root_case() {
        let path = HttpPath::new("/");

        assert!(path.is_root());
        assert_eq!(0, path.segments_amount());
    }

    #[test]
    fn test_one_segment_case_not_slash_at_the_end() {
        let path = HttpPath::new("/First");

        assert!(!path.is_root());
        assert_eq!(1, path.segments_amount());
        assert_eq!("First", path.get_segment_value(0).unwrap());
        assert!(path.get_segment_value(1).is_none());
    }

    #[test]
    fn test_one_segment_case_with_slash_at_the_end() {
        let path = HttpPath::new("/first/");

        assert!(!path.is_root());
        assert_eq!(1, path.segments_amount());
        assert_eq!("first", path.get_segment_value(0).unwrap());
        assert!(path.get_segment_value(1).is_none());
    }

    #[test]
    fn test_two_segments_case_not_slash_at_the_end() {
        let path = HttpPath::new("/First/sEcond");

        assert!(!path.is_root());
        assert_eq!(2, path.segments_amount());
        assert_eq!("First", path.get_segment_value(0).unwrap());
        assert_eq!("sEcond", path.get_segment_value(1).unwrap());
        assert!(path.get_segment_value(2).is_none());
    }

    #[test]
    fn test_two_segment_case_with_slash_at_the_end() {
        let path = HttpPath::new("/first/second/");

        assert!(!path.is_root());
        assert_eq!(2, path.segments_amount());
        assert_eq!("first", path.get_segment_value(0).unwrap());
        assert_eq!("second", path.get_segment_value(1).unwrap());
        assert!(path.get_segment_value(2).is_none());
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_my_path() {
        let path_segments = HttpPath::new("/Segment1/{Key1}/Segment2");

        assert_eq!(true, path_segments.is_my_path("/Segment1/MyValue/Segment2"));

        assert_eq!(
            "MyValue",
            path_segments
                .get_value("/Segment1/MyValue/Segment2", "key1")
                .unwrap()
                .as_str()
        );
    }

    #[test]
    fn test_is_not_my_path() {
        let path_segments = HttpPath::new("/Segment1/{Key1}/Segment2");

        assert!(path_segments.is_my_path("/Segment2/MyValue/Segment2"));
    }

    #[test]
    fn test_is_my_path_with_last_key() {
        let path_segments = HttpPath::new("/Segment1/Segment2/{Key1}");

        assert_eq!(true, path_segments.is_my_path("/Segment1/Segment2"));

        let value = path_segments.get_value("/Segment1/Segment2", "Key1");

        assert_eq!(true, value.is_none());
    }
}
 */
