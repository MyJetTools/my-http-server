pub struct CookiesReader<'s> {
    pub src: Option<&'s str>,
}

impl<'s> CookiesReader<'s> {
    pub fn new(src: Option<&'s str>) -> Self {
        Self { src }
    }

    pub fn get(&'s self, name_to_find: &str) -> Option<&'s str> {
        for (name, value) in self.iterate_all() {
            if name == name_to_find {
                return Some(value);
            }
        }

        None
    }

    pub fn iterate_all(&'s self) -> Vec<(&'s str, &'s str)> {
        let Some(src) = self.src else {
            return vec![];
        };

        let mut result = Vec::new();

        for kv in src.split(';') {
            let index = kv.find('=');

            match index {
                Some(index) => {
                    result.push((&kv[..index], &kv[index + 1..]));
                }
                None => {
                    result.push((kv, ""));
                }
            }
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
