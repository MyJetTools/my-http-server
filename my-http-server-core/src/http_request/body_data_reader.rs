use crate::{url_encoded_data::UrlEncodedData, EncodedParamValue, HttpFailResult, JsonEncodedData};

use crate::data_src::*;
pub enum BodyDataReaderInner<'s> {
    UrlEncoded(UrlEncodedData<'s>),
    JsonEncoded(JsonEncodedData<'s>),
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

    pub fn get_required(
        &'s self,
        name: &'static str,
    ) -> Result<EncodedParamValue<'s>, HttpFailResult> {
        match &self.inner {
            BodyDataReaderInner::UrlEncoded(src) => src.get_required(name),
            BodyDataReaderInner::JsonEncoded(src) => {
                let value = src.get_required(name)?;
                Ok(EncodedParamValue::JsonEncodedData {
                    name: name,
                    value,
                    src: SRC_BODY_JSON,
                })
            }
        }
    }

    pub fn get_optional(&'s self, name: &'static str) -> Option<EncodedParamValue<'s>> {
        match &self.inner {
            BodyDataReaderInner::UrlEncoded(result) => result.get_optional(name),
            BodyDataReaderInner::JsonEncoded(result) => {
                let value = result.get_optional(name)?;
                Some(EncodedParamValue::JsonEncodedData {
                    name: name.into(),
                    value,
                    src: SRC_BODY_JSON,
                })
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

    #[test]
    fn test_boolean() {
        use crate::{BodyDataReader, JsonEncodedData};

        let src_data = r#"{"name":"John","age":30,"cars":["Ford","BMW","Fiat"],"is_admin":true, ,"is_user":false}"#;

        let json_encoded_data = JsonEncodedData::new(src_data.as_bytes()).unwrap();
        let body_data_reader = BodyDataReader::create_as_json_encoded_data(json_encoded_data);

        let result: bool = body_data_reader
            .get_required("is_admin")
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(true, result);

        let result: bool = body_data_reader
            .get_required("is_user")
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(false, result);
    }
}