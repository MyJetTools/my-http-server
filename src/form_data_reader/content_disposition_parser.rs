#[derive(Debug)]
pub struct KeyValue<'s> {
    pub key: &'s str,
    pub value: Option<&'s str>,
}

pub struct ContentDispositionParser<'s> {
    content: &'s [u8],
    pos: usize,
}

impl<'s> ContentDispositionParser<'s> {
    pub fn new(content: &'s [u8]) -> Self {
        Self { content, pos: 0 }
    }
    pub fn find_pos<TCondition: Fn(u8) -> bool>(&self, condition: TCondition) -> Option<usize> {
        for i in self.pos..self.content.len() {
            if condition(self.content[i]) {
                return Some(i);
            }
        }

        None
    }
}

impl<'s> Iterator for ContentDispositionParser<'s> {
    type Item = KeyValue<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos = self.find_pos(|b| b > 32)?;

        let pos = self.find_pos(|b| b == b';' || b == b'=')?;

        let b = self.content[pos];

        let key = std::str::from_utf8(&self.content[self.pos..pos]).unwrap();
        self.pos = pos + 1;

        if b == b';' {
            let result = KeyValue { key, value: None };
            return Some(result);
        }

        let start_of_value = self.find_pos(|b| b == b'"')?;

        self.pos = start_of_value + 1;

        let end_of_value = self.find_pos(|b| b == b'"')?;

        let value = if self.content[start_of_value] == b'"' {
            let result = &self.content[start_of_value + 1..end_of_value];
            self.pos = end_of_value + 2;
            result
        } else {
            let result = &self.content[start_of_value..end_of_value + 1];
            self.pos = end_of_value + 1;
            result
        };

        Some(KeyValue {
            key,
            value: Some(std::str::from_utf8(value).unwrap()),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_content_disposition_as_field() {
        let src: Vec<u8> = vec![
            32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61, 34, 100,
            116, 70, 114, 111, 109, 34, 13, 10,
        ];

        let result = ContentDispositionParser::new(&src).collect::<Vec<_>>();

        {
            let first_item = result.get(0).unwrap();
            assert_eq!(first_item.key, "form-data");
            assert_eq!(first_item.value, None);
        }

        {
            let first_item = result.get(1).unwrap();
            assert_eq!(first_item.key, "name");
            assert_eq!(first_item.value, Some("dtFrom"));
        }
    }

    #[test]
    pub fn test_content_disposition_as_file() {
        let src: Vec<u8> = vec![
            32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61, 34, 102,
            105, 108, 101, 34, 59, 32, 102, 105, 108, 101, 110, 97, 109, 101, 61, 34, 116, 101,
            115, 116, 45, 112, 97, 121, 108, 111, 97, 100, 46, 116, 120, 116, 34, 13, 10, 67, 111,
            110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 116, 101, 120, 116, 47, 112,
            108, 97, 105, 110, 13, 10, 13, 10, 49, 50, 51, 13, 10,
        ];

        let result = ContentDispositionParser::new(&src).collect::<Vec<_>>();

        {
            let first_item = result.get(0).unwrap();
            assert_eq!(first_item.key, "form-data");
            assert_eq!(first_item.value, None);
        }

        {
            let first_item = result.get(1).unwrap();
            assert_eq!(first_item.key, "name");
            assert_eq!(first_item.value, Some("file"));
        }

        {
            let first_item = result.get(2).unwrap();
            assert_eq!(first_item.key, "filename");
            assert_eq!(first_item.value, Some("test-payload.txt"));
        }
    }
}
