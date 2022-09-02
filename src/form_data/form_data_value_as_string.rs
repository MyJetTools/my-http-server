use std::str::FromStr;

use url_utils::url_encoded_data_reader::UrlEncodedValueAsString;

use crate::{json_encoded_data::JsonEncodedValueAsString, HttpFailResult};

pub enum FormDataValueAsString<'s> {
    UrlEncodedValueAsString(&'s UrlEncodedValueAsString<'s>),
    JsonEncodedData(&'s JsonEncodedValueAsString<'s>),
}

impl<'s> FormDataValueAsString<'s> {
    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self {
            FormDataValueAsString::UrlEncodedValueAsString(result) => {
                let result = result.as_string();
                return crate::url_encoded_data::convert_error(
                    result,
                    crate::UrlEncodedDataSource::FormData,
                );
            }

            FormDataValueAsString::JsonEncodedData(result) => result.as_string(),
        }
    }
    pub fn as_bool(&self) -> Result<bool, HttpFailResult> {
        match self {
            FormDataValueAsString::UrlEncodedValueAsString(result) => {
                let result = result.as_bool();
                return crate::url_encoded_data::convert_error(
                    result,
                    crate::UrlEncodedDataSource::FormData,
                );
            }
            FormDataValueAsString::JsonEncodedData(result) => {
                return result.as_bool();
            }
        }
    }
    pub fn parse<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self {
            FormDataValueAsString::UrlEncodedValueAsString(result) => {
                let result = result.parse();
                return crate::url_encoded_data::convert_error(
                    result,
                    crate::UrlEncodedDataSource::FormData,
                );
            }
            FormDataValueAsString::JsonEncodedData(result) => result.parse(),
        }
    }
}
