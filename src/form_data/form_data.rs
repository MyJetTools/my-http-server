use crate::{url_encoded_data::UrlEncodedData, HttpFailResult, JsonEncodedData};

use super::FormDataValueAsString;

pub enum FormDataInner<'s> {
    UrlEncoded(UrlEncodedData<'s>),
    JsonEncoded(JsonEncodedData<'s>),
}
pub struct FormData<'s> {
    inner: FormDataInner<'s>,
}

impl<'s> FormData<'s> {
    pub fn crate_as_url_encoded_data(src: UrlEncodedData<'s>) -> Self {
        Self {
            inner: FormDataInner::UrlEncoded(src),
        }
    }

    pub fn create_as_json_encoded_data(src: JsonEncodedData<'s>) -> Self {
        Self {
            inner: FormDataInner::JsonEncoded(src),
        }
    }

    pub fn get_required(
        &'s self,
        name: &'s str,
    ) -> Result<FormDataValueAsString<'s>, HttpFailResult> {
        match &self.inner {
            FormDataInner::UrlEncoded(result) => {
                let result = result.get_required(name)?;
                Ok(FormDataValueAsString::UrlEncodedValueAsString(result))
            }
            FormDataInner::JsonEncoded(result) => {
                let result = result.get_required(name)?;
                Ok(FormDataValueAsString::JsonEncodedData(result))
            }
        }
    }

    pub fn get_optional(&'s self, name: &'s str) -> Option<FormDataValueAsString<'s>> {
        match &self.inner {
            FormDataInner::UrlEncoded(result) => {
                let result = result.get_optional(name)?;
                Some(FormDataValueAsString::UrlEncodedValueAsString(result))
            }
            FormDataInner::JsonEncoded(result) => {
                let result = result.get_optional(name)?;
                Some(FormDataValueAsString::JsonEncodedData(result))
            }
        }
    }
}
