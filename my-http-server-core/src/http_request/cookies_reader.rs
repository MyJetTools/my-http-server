pub struct CookiesReader<'s> {
    pub src: Option<&'s str>,
}

impl<'s> CookiesReader<'s> {
    pub fn new(src: Option<&'s str>) -> Self {
        Self { src }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        let src = self.src?;

        for kvp in src.split(';') {
            let kvp = kvp.trim();

            let mut kv = kvp.split("=");

            let key = kv.next().unwrap();

            if key == name {
                return kv.next();
            }
        }

        None
    }

    pub fn iterate_all(&'s self) -> Vec<(&'s str, String)> {
        if self.src.is_none() {
            return vec![];
        }

        let mut result = Vec::new();

        for kv in self.src.unwrap().split(';') {
            let mut kv = kv.split("=");

            let key = kv.next().unwrap();
            let value =
                url_utils::url_decoder::decode_as_str_or_string(kv.next().unwrap_or_default());

            let value = match value {
                Ok(value) => value.to_string(),
                Err(_) => String::new(),
            };

            result.push((key, value));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::CookiesReader;

    #[test]
    fn test_basic_cases() {
        let header_value = "_octo=GH1.1.17; _device_id=7763; saved_user_sessions=527071";

        let reader = CookiesReader::new(header_value.into());

        assert_eq!(reader.get("_octo"), Some("GH1.1.17"));
        assert_eq!(reader.get("_device_id"), Some("7763"));
        assert_eq!(reader.get("saved_user_sessions"), Some("527071"));
        assert_eq!(reader.get("not_found"), None);
    }
}
