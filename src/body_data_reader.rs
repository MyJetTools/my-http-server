use crate::{url_encoded_data::UrlEncodedData, HttpFailResult, JsonEncodedData};

use super::ValueAsString;

pub const BODY_SRC: &str = "body";

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

    pub fn get_required(&'s self, name: &'s str) -> Result<ValueAsString<'s>, HttpFailResult> {
        match &self.inner {
            BodyDataReaderInner::UrlEncoded(result) => {
                let value = result.get_required(name)?;
                Ok(ValueAsString::UrlEncodedValueAsStringRef {
                    value,
                    src: BODY_SRC,
                })
            }
            BodyDataReaderInner::JsonEncoded(result) => {
                let value = result.get_required(name)?;
                Ok(ValueAsString::JsonEncodedData {
                    value,
                    src: BODY_SRC,
                })
            }
        }
    }

    pub fn get_optional(&'s self, name: &'s str) -> Option<ValueAsString<'s>> {
        match &self.inner {
            BodyDataReaderInner::UrlEncoded(result) => {
                let value = result.get_optional(name)?;
                Some(ValueAsString::UrlEncodedValueAsStringRef {
                    value,
                    src: BODY_SRC,
                })
            }
            BodyDataReaderInner::JsonEncoded(result) => {
                let value = result.get_optional(name)?;
                Some(ValueAsString::JsonEncodedData {
                    value,
                    src: BODY_SRC,
                })
            }
        }
    }
}
