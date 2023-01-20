use crate::{
    types::FileContent, url_encoded_data::UrlEncodedData, FormDataReader, HttpFailResult,
    JsonEncodedData,
};

use super::InputParamValue;

pub const BODY_JSON_SRC: &str = "body json";
pub const BODY_URL_SRC: &str = "body url/encoded";
pub const FORM_DATA_SRC: &str = "body url/encoded";

pub enum BodyDataReaderInner<'s> {
    UrlEncoded(UrlEncodedData<'s>),
    JsonEncoded(JsonEncodedData<'s>),
    FormData(FormDataReader<'s>),
}
pub struct BodyDataReader<'s> {
    inner: BodyDataReaderInner<'s>,
}

impl<'s> BodyDataReader<'s> {
    pub fn crate_as_url_encoded_data(src: UrlEncodedData<'s>) -> Self {
        Self {
            inner: BodyDataReaderInner::UrlEncoded(src),
        }
    }

    pub fn create_as_json_encoded_data(src: JsonEncodedData<'s>) -> Self {
        Self {
            inner: BodyDataReaderInner::JsonEncoded(src),
        }
    }

    pub fn create_as_form_data(body: &'s [u8]) -> Self {
        Self {
            inner: BodyDataReaderInner::FormData(FormDataReader::new(body)),
        }
    }

    pub fn get_required(&'s self, name: &'s str) -> Result<InputParamValue<'s>, HttpFailResult> {
        match &self.inner {
            BodyDataReaderInner::UrlEncoded(src) => {
                let value = src.get_required(name)?;
                Ok(InputParamValue::UrlEncodedValueAsStringRef {
                    value,
                    src: BODY_URL_SRC,
                })
            }
            BodyDataReaderInner::JsonEncoded(src) => {
                let value = src.get_required(name)?;
                Ok(InputParamValue::JsonEncodedData {
                    value,
                    src: BODY_JSON_SRC,
                })
            }
            BodyDataReaderInner::FormData(src) => {
                let item = src.get_required(name)?;
                match item {
                    crate::FormDataItem::ValueAsString { value, name: _ } => {
                        return Ok(InputParamValue::Raw {
                            value,
                            src: "form data",
                        })
                    }
                    crate::FormDataItem::File {
                        name: _,
                        file_name,
                        content_type,
                        content,
                    } => {
                        return Ok(InputParamValue::File {
                            file: FileContent {
                                content_type: content_type.to_string(),
                                file_name: file_name.to_string(),
                                content: content.to_vec(),
                            },
                            src: "form data",
                        });
                    }
                }
            }
        }
    }

    pub fn get_optional(&'s self, name: &'s str) -> Option<InputParamValue<'s>> {
        match &self.inner {
            BodyDataReaderInner::UrlEncoded(result) => {
                let value = result.get_optional(name)?;
                Some(InputParamValue::UrlEncodedValueAsStringRef {
                    value,
                    src: BODY_URL_SRC,
                })
            }
            BodyDataReaderInner::JsonEncoded(result) => {
                let value = result.get_optional(name)?;
                Some(InputParamValue::JsonEncodedData {
                    value,
                    src: BODY_JSON_SRC,
                })
            }
            BodyDataReaderInner::FormData(src) => {
                let item = src.get_optional(name)?;

                match item {
                    crate::FormDataItem::ValueAsString { value, name: _ } => {
                        return Some(InputParamValue::Raw {
                            value,
                            src: "form data",
                        })
                    }
                    crate::FormDataItem::File {
                        name: _,
                        file_name,
                        content_type,
                        content,
                    } => {
                        return Some(InputParamValue::File {
                            file: FileContent {
                                content_type: content_type.to_string(),
                                file_name: file_name.to_string(),
                                content: content.to_vec(),
                            },
                            src: "form data",
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_parsing_json_model_with_array_as_param() {
        use crate::{BodyDataReader, JsonEncodedData};

        let src_data = r#"{"name":"John","age":30,"cars":["Ford","BMW","Fiat"]}"#;

        let json_encoded_data = JsonEncodedData::new(src_data.as_bytes()).unwrap();
        let body_data_reader = BodyDataReader::create_as_json_encoded_data(json_encoded_data);

        assert_eq!(
            "John",
            body_data_reader
                .get_required("name")
                .unwrap()
                .as_string()
                .unwrap()
        );

        let result: i32 = body_data_reader
            .get_required("age")
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(30, result);

        let result: Vec<String> = body_data_reader
            .get_required("cars")
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(vec!["Ford", "BMW", "Fiat"], result);
    }
}
