use rust_extensions::{MaybeShortString, ShortString};

pub struct HttpPathReader<'s> {
    data: &'s str,
}

impl<'s> HttpPathReader<'s> {
    pub fn new(data: &'s str) -> Self {
        Self { data }
    }
    pub fn as_str(&self) -> &str {
        self.data
    }

    pub fn to_string(&self) -> String {
        self.data.to_string()
    }

    pub fn equals_to(&self, path: &str) -> bool {
        self.data == path
    }

    pub fn equals_to_case_insensitive(&self, path: &str) -> bool {
        if path.len() != self.data.len() {
            return false;
        }

        let mut self_chars = self.data.chars();

        for path_char in path.chars() {
            let next_char = self_chars.next();

            if next_char.is_none() {
                return false;
            }

            let next_char = next_char.unwrap();

            if next_char.to_ascii_lowercase() != path_char.to_ascii_lowercase() {
                return false;
            }
        }

        true
    }

    pub fn to_short_string_or_string(&self) -> MaybeShortString {
        let as_bytes = self.data.as_bytes();

        if as_bytes.len() <= 255 {
            return MaybeShortString::AsShortString(ShortString::from_str(&self.data).unwrap());
        }

        MaybeShortString::AsString(self.data.to_string())
    }

    pub fn starts_with_case_insensitive(&self, path: &str) -> bool {
        if path.len() > self.data.len() {
            return false;
        }

        let mut self_chars = self.data.chars();

        for path_char in path.chars() {
            let next_char = self_chars.next();

            if next_char.is_none() {
                return false;
            }

            let next_char = next_char.unwrap();

            if next_char.to_ascii_lowercase() != path_char.to_ascii_lowercase() {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::HttpPathReader;

    #[test]
    fn test_starts_with_case_insensitive() {
        let path = "/MyPath/Other";

        let reader = HttpPathReader::new(path);

        assert_eq!(true, reader.starts_with_case_insensitive("/mypath"));
        assert_eq!(true, reader.starts_with_case_insensitive("/MyPath"));
    }

    #[test]
    fn test_starts_with_case_insensitive_exact_string() {
        let path = "/MyPath";

        let reader = HttpPathReader::new(path);

        assert_eq!(true, reader.starts_with_case_insensitive("/mypath"));
        assert_eq!(true, reader.starts_with_case_insensitive("/MyPath"));
    }

    #[test]
    fn test_starts_with_case_insensitive_less_then_we_match() {
        let path = "/MyPat";

        let reader = HttpPathReader::new(path);

        assert_eq!(false, reader.starts_with_case_insensitive("/mypath"));
        assert_eq!(false, reader.starts_with_case_insensitive("/MyPath"));
    }

    #[test]
    fn test_equals_to_case_insensitive() {
        let path = "/MyPath";

        let reader = HttpPathReader::new(path);

        assert_eq!(true, reader.equals_to_case_insensitive("/mypath"));
        assert_eq!(true, reader.equals_to_case_insensitive("/MyPath"));

        assert_eq!(false, reader.equals_to_case_insensitive("/mypat"));
        assert_eq!(false, reader.equals_to_case_insensitive("/MyPatht"));
        assert_eq!(false, reader.equals_to_case_insensitive("/mypatc"));
    }
}
