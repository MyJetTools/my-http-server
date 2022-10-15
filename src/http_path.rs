#[derive(Debug, Clone)]
pub struct HttpPath {
    pub path: Vec<u8>,
    segments: Vec<usize>,
}

impl HttpPath {
    pub fn from_str(path: &str) -> Self {
        Self::from_vec(path.as_bytes().to_vec())
    }
    pub fn from_string(path: String) -> Self {
        Self::from_vec(path.into_bytes())
    }

    pub fn from_vec(path_as_vec: Vec<u8>) -> Self {
        let mut segments = Vec::new();

        if path_as_vec.len() == 0 {
            panic!("Invalid http path. Path at least should contain one character '/'");
        }

        if path_as_vec[0] != b'/' {
            panic!("Invalid http path. Path should start with '/'");
        }

        let mut last: u8 = 0;

        for pos in 0..path_as_vec.len() {
            last = path_as_vec[pos];
            if last == b'/' {
                segments.push(pos);
            }
        }

        if last != b'/' {
            segments.push(path_as_vec.len());
        }

        Self {
            path: path_as_vec,
            segments,
        }
    }

    pub fn is_the_same_to(&self, http_path: &HttpPath) -> bool {
        let segments_amount = self.segments_amount();
        if segments_amount != http_path.segments_amount() {
            return false;
        }

        for i in 0..segments_amount {
            let src = self.get_segment_value(i).unwrap();
            let dest = http_path.get_segment_value(i).unwrap();
            if !equal_strings_case_insensitive(src, dest) {
                return false;
            }
        }

        return true;
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

    pub fn has_value_at_index_case_insensitive(&self, index: usize, value: &str) -> bool {
        if let Some(segment_value) = self.get_segment_value(index) {
            return equal_strings_case_insensitive(segment_value, value);
        }

        false
    }

    pub fn has_values_at_index_case_insensitive(&self, index_from: usize, values: &[&str]) -> bool {
        for offset in 0..values.len() {
            let value = values.get(offset).unwrap();
            if let Some(segment_value) = self.get_segment_value(index_from + offset) {
                if !equal_strings_case_insensitive(segment_value, value) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

fn equal_strings_case_insensitive(src: &str, dest: &str) -> bool {
    if src.len() != dest.len() {
        return false;
    }

    for (src_char, dest_char) in src.chars().zip(dest.chars()) {
        if src_char.to_ascii_lowercase() != dest_char.to_ascii_lowercase() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_root_case() {
        let path = HttpPath::from_str("/");

        assert!(path.is_root());
        assert_eq!(0, path.segments_amount());
    }

    #[test]
    fn test_one_segment_case_not_slash_at_the_end() {
        let path = HttpPath::from_str("/First");

        assert!(!path.is_root());
        assert_eq!(1, path.segments_amount());
        assert_eq!("First", path.get_segment_value(0).unwrap());
        assert!(path.get_segment_value(1).is_none());
    }

    #[test]
    fn test_one_segment_case_with_slash_at_the_end() {
        let path = HttpPath::from_str("/first/");

        assert!(!path.is_root());
        assert_eq!(1, path.segments_amount());
        assert_eq!("first", path.get_segment_value(0).unwrap());
        assert!(path.get_segment_value(1).is_none());
    }

    #[test]
    fn test_two_segments_case_not_slash_at_the_end() {
        let path = HttpPath::from_str("/First/sEcond");

        assert!(!path.is_root());
        assert_eq!(2, path.segments_amount());
        assert_eq!("First", path.get_segment_value(0).unwrap());
        assert_eq!("sEcond", path.get_segment_value(1).unwrap());
        assert!(path.get_segment_value(2).is_none());
    }

    #[test]
    fn test_two_segment_case_with_slash_at_the_end() {
        let path = HttpPath::from_str("/first/second/");

        assert!(!path.is_root());
        assert_eq!(2, path.segments_amount());
        assert_eq!("first", path.get_segment_value(0).unwrap());
        assert_eq!("second", path.get_segment_value(1).unwrap());
        assert!(path.get_segment_value(2).is_none());
    }

    #[test]
    fn test_segment_by_segment() {
        let path = HttpPath::from_str("/First/Second/");

        assert!(path.has_value_at_index_case_insensitive(0, "first"));
        assert!(path.has_value_at_index_case_insensitive(1, "second"));
    }

    #[test]
    fn test_segments() {
        let path = HttpPath::from_str("/First/Second/");

        assert!(path.has_values_at_index_case_insensitive(0, &["first", "second"]));

        assert!(!path.has_values_at_index_case_insensitive(1, &["second", "third"]));
    }

    #[test]
    fn test_paths_equality() {
        let path1 = HttpPath::from_str("/First/Second/");
        let path2 = HttpPath::from_str("/first/second/");

        assert!(path1.is_the_same_to(&path2));

        let path2 = HttpPath::from_str("/first/second");

        assert!(path1.is_the_same_to(&path2));
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
