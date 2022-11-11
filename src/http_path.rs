use url_utils::url_encoded_data_reader::UrlEncodedValueAsString;

use crate::ValueAsString;

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
            let src = self.get_segment_value_as_str(i).unwrap();
            let dest = http_path.get_segment_value_as_str(i).unwrap();
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

    pub fn get_segment_value_as_str(&self, index: usize) -> Option<&str> {
        let pos = self.segments.get(index + 1)?;
        let pos_prev = self.segments.get(index)?;

        let result = &self.path[*pos_prev + 1..*pos];
        Some(std::str::from_utf8(result).unwrap())
    }

    pub fn get_segment_value(&self, index: usize) -> Option<ValueAsString> {
        let value = self.get_segment_value_as_str(index)?;
        Some(ValueAsString::UrlEncodedValueAsString {
            value: UrlEncodedValueAsString::new(value),
            src: "path",
        })
    }

    pub fn is_starting_with(&self, http_path: &HttpPath) -> bool {
        if self.segments_amount() < http_path.segments_amount() {
            return false;
        }

        for index in 0..http_path.segments_amount() {
            let one_side = http_path.get_segment_value_as_str(index).unwrap();
            let other_side = self.get_segment_value_as_str(index).unwrap();
            if !equal_strings_case_insensitive(one_side, other_side) {
                return false;
            }
        }

        true
    }

    pub fn as_str_from_segment(&self, from_segment: usize) -> &str {
        let result = &self.path[self.segments[from_segment]..];

        std::str::from_utf8(result).unwrap()
    }

    pub fn has_value_at_index_case_insensitive(&self, index: usize, value: &str) -> bool {
        if let Some(segment_value) = self.get_segment_value_as_str(index) {
            return equal_strings_case_insensitive(segment_value, value);
        }

        false
    }

    pub fn has_values_at_index_case_insensitive(&self, index_from: usize, values: &[&str]) -> bool {
        for offset in 0..values.len() {
            let value = values.get(offset).unwrap();
            if let Some(segment_value) = self.get_segment_value_as_str(index_from + offset) {
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
        assert_eq!("First", path.get_segment_value_as_str(0).unwrap());
        assert!(path.get_segment_value(1).is_none());
    }

    #[test]
    fn test_one_segment_case_with_slash_at_the_end() {
        let path = HttpPath::from_str("/first/");

        assert!(!path.is_root());
        assert_eq!(1, path.segments_amount());
        assert_eq!("first", path.get_segment_value_as_str(0).unwrap());
        assert!(path.get_segment_value(1).is_none());
    }

    #[test]
    fn test_two_segments_case_not_slash_at_the_end() {
        let path = HttpPath::from_str("/First/sEcond");

        assert!(!path.is_root());
        assert_eq!(2, path.segments_amount());
        assert_eq!("First", path.get_segment_value_as_str(0).unwrap());
        assert_eq!("sEcond", path.get_segment_value_as_str(1).unwrap());
        assert!(path.get_segment_value(2).is_none());
    }

    #[test]
    fn test_two_segment_case_with_slash_at_the_end() {
        let path = HttpPath::from_str("/first/second/");

        assert!(!path.is_root());
        assert_eq!(2, path.segments_amount());
        assert_eq!("first", path.get_segment_value_as_str(0).unwrap());
        assert_eq!("second", path.get_segment_value_as_str(1).unwrap());
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

    #[test]
    fn test_starts_with() {
        let src = HttpPath::from_str("/First/Second/Third");

        let path2 = HttpPath::from_str("/first");
        assert!(src.is_starting_with(&path2));

        let path2 = HttpPath::from_str("/first/second/");
        assert!(src.is_starting_with(&path2));

        let path2 = HttpPath::from_str("/first/second/third");
        assert!(src.is_starting_with(&path2));

        let path2 = HttpPath::from_str("/first/second/third/fourth");
        assert!(!src.is_starting_with(&path2));

        let path2 = HttpPath::from_str("/ffirst/second");
        assert!(!src.is_starting_with(&path2));
    }

    #[test]
    fn test_as_str_from_segment() {
        let src = HttpPath::from_str("/First/Second/Third");
        let result = src.as_str_from_segment(0);
        assert_eq!("/First/Second/Third", result);

        let result = src.as_str_from_segment(1);
        assert_eq!("/Second/Third", result);

        let result = src.as_str_from_segment(2);
        assert_eq!("/Third", result);
    }
}
