use crate::ContentDispositionParser;

#[derive(Debug)]
pub enum FormDataItem<'s> {
    ValueAsString {
        value: &'s str,
        name: &'s str,
    },
    File {
        name: &'s str,
        file_name: &'s str,
        content_type: &'s str,
        content: &'s [u8],
    },
}

impl<'s> FormDataItem<'s> {
    pub fn unwrap_as_string(&'s self) -> &'s str {
        match self {
            FormDataItem::ValueAsString { value, .. } => value,
            FormDataItem::File { .. } => {
                panic!("Can not unwrap FormDataItem as string. It is file")
            }
        }
    }

    pub fn unwrap_as_file_name(&'s self) -> &'s str {
        match self {
            FormDataItem::ValueAsString { .. } => {
                panic!("Can not unwrap FormDataItem as string. It is file")
            }
            FormDataItem::File { file_name, .. } => file_name,
        }
    }

    pub fn get_name(&'s self) -> &'s str {
        match self {
            FormDataItem::ValueAsString { name, .. } => name,
            FormDataItem::File { name, .. } => name,
        }
    }
    pub fn parse(src: &'s [u8]) -> Self {
        let mut content_type = None;
        let content;
        let mut file_name = None;
        let mut name = None;

        let mut pos = 0;

        loop {
            if &src[pos..pos + 4] == &[13u8, 10u8, 13u8, 10u8] {
                pos += 4;

                content = Some(&src[pos..src.len() - 2]);
                break;
            }

            pos = rust_extensions::slice_of_u8_utils::find_pos_by_condition(src, pos, |p| p > 32)
                .unwrap();

            let double_quote_pos =
                rust_extensions::slice_of_u8_utils::find_byte_pos(src, b':' as u8, pos);

            if double_quote_pos.is_none() {
                panic!("Invalid form data parsing. Can not find ':'");
            }

            let double_quote_pos = double_quote_pos.unwrap();

            let header_name = std::str::from_utf8(&src[pos..double_quote_pos]).unwrap();

            match header_name {
                "Content-Disposition" => {
                    let content_disposition_data = &src[double_quote_pos + 1..];

                    let end = content_disposition_data
                        .iter()
                        .position(|p| *p == 13)
                        .unwrap();

                    let content_disposition_data = &content_disposition_data[..end];

                    for itm in ContentDispositionParser::new(content_disposition_data) {
                        match itm.key {
                            "name" => name = itm.value,
                            "filename" => file_name = itm.value,
                            _ => {}
                        }
                    }

                    pos = rust_extensions::slice_of_u8_utils::find_byte_pos(
                        src,
                        13,
                        double_quote_pos,
                    )
                    .unwrap();
                }
                "Content-Type" => {
                    let start = rust_extensions::slice_of_u8_utils::find_pos_by_condition(
                        src,
                        double_quote_pos + 1,
                        |p| p > 32,
                    )
                    .unwrap();
                    let end = rust_extensions::slice_of_u8_utils::find_pos_by_condition(
                        src,
                        start,
                        |p| p == 13,
                    )
                    .unwrap();
                    content_type = Some(std::str::from_utf8(&src[start..end]).unwrap());

                    pos = end;
                }
                _ => {
                    pos = rust_extensions::slice_of_u8_utils::find_byte_pos(
                        src,
                        13,
                        double_quote_pos,
                    )
                    .unwrap();
                }
            }
        }

        if let Some(content_type) = content_type {
            Self::File {
                file_name: file_name.unwrap(),
                content_type: content_type,
                content: content.unwrap(),
                name: name.unwrap(),
            }
        } else {
            Self::ValueAsString {
                value: std::str::from_utf8(content.unwrap()).unwrap(),
                name: name.unwrap(),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_value_as_string_parser() {
        let msg = vec![
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 100, 116, 70, 114, 111, 109, 34, 13, 10, 13, 10, 50, 13, 10,
        ];

        let item = super::FormDataItem::parse(&msg);

        match item {
            crate::FormDataItem::ValueAsString { value, name } => {
                assert_eq!(name, "dtFrom");
                assert_eq!(value, "2");
            }
            crate::FormDataItem::File { .. } => {
                panic!("Should be value as string");
            }
        }
    }

    #[test]
    fn test_value_file_parser() {
        let msg = vec![
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 102, 105, 108, 101, 34, 59, 32, 102, 105, 108, 101, 110, 97, 109, 101, 61, 34, 116,
            101, 115, 116, 45, 112, 97, 121, 108, 111, 97, 100, 46, 116, 120, 116, 34, 13, 10, 67,
            111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 116, 101, 120, 116, 47,
            112, 108, 97, 105, 110, 13, 10, 13, 10, 49, 50, 51, 13, 10,
        ];

        let item = super::FormDataItem::parse(&msg);

        match item {
            crate::FormDataItem::ValueAsString { value: _, name: _ } => {
                panic!("Should be value as string");
            }
            crate::FormDataItem::File {
                name,
                file_name,
                content_type,
                content,
            } => {
                assert_eq!(name, "file");
                assert_eq!(file_name, "test-payload.txt");
                assert_eq!(content_type, "text/plain");
                assert_eq!(std::str::from_utf8(content).unwrap(), "123");
            }
        }
    }

    #[test]
    pub fn test_content_disposition_test_real_data() {
        let src: Vec<u8> = vec![
            67, 111, 110, 116, 101, 110, 116, 45, 68, 105, 115, 112, 111, 115, 105, 116, 105, 111,
            110, 58, 32, 102, 111, 114, 109, 45, 100, 97, 116, 97, 59, 32, 110, 97, 109, 101, 61,
            34, 100, 111, 99, 73, 100, 34, 13, 10, 13, 10, 48, 13, 10,
        ];

        println!("src: {:?}", std::str::from_utf8(src.as_slice()).unwrap());

        let result = super::FormDataItem::parse(&src);

        assert_eq!(result.get_name(), "docId");
        assert_eq!(result.unwrap_as_string(), "0");
    }
    
}
