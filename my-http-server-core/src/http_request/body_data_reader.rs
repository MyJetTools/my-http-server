use crate::{url_encoded_data::UrlEncodedData, EncodedParamValue, HttpFailResult, JsonEncodedData};

use crate::{data_src::*, FormDataReader};

pub enum BodyDataReader<'s> {
    UrlEncoded(UrlEncodedData<'s>),
    JsonEncoded(JsonEncodedData<'s>),
    FormData(FormDataReader<'s>),
    Unknown(&'s [u8]),
    Empty,
}

impl<'s> BodyDataReader<'s> {
    pub fn crate_as_url_encoded_data(src: UrlEncodedData<'s>) -> Self {
        Self::UrlEncoded(src)
    }

    pub fn create_as_json_encoded_data(src: JsonEncodedData<'s>) -> Self {
        Self::JsonEncoded(src)
    }

    pub fn create_as_form_data_reader(src: FormDataReader<'s>) -> Self {
        Self::FormData(src)
    }

    pub fn get_required(
        &'s self,
        name: &'static str,
    ) -> Result<EncodedParamValue<'s>, HttpFailResult> {
        match self {
            Self::UrlEncoded(src) => src.get_required(name),
            Self::JsonEncoded(src) => {
                let value = src.get_required(name)?;
                Ok(EncodedParamValue::JsonEncodedData {
                    name: name,
                    value,
                    src: SRC_BODY_JSON,
                })
            }
            Self::FormData(src) => {
                let result = src.get_required(name)?;
                Ok(EncodedParamValue::FormData {
                    name,
                    value: result,
                })
            }

            Self::Unknown(_) => Err(HttpFailResult::as_validation_error(
                "Body has unknown format. Can not read data from it".to_string(),
            )),

            Self::Empty => Err(HttpFailResult::as_validation_error(
                "Body is empty. Can not read data from it".to_string(),
            )),
        }
    }

    pub fn get_optional(&'s self, name: &'static str) -> Option<EncodedParamValue<'s>> {
        match self {
            Self::UrlEncoded(result) => result.get_optional(name),
            Self::JsonEncoded(result) => {
                let value = result.get_optional(name)?;
                Some(EncodedParamValue::JsonEncodedData {
                    name: name.into(),
                    value,
                    src: SRC_BODY_JSON,
                })
            }

            Self::FormData(src) => {
                let value = src.get_optional(name)?;
                Some(EncodedParamValue::FormData { name, value })
            }

            BodyDataReader::Unknown(_) => None,

            BodyDataReader::Empty => None,
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

        let src_data = r#"{"name":"John","age":30,"cars":["Ford","BMW","Fiat"],"is_admin":true, "is_user":false}"#;

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
