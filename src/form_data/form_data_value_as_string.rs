use std::str::FromStr;

use crate::{
    json_encoded_data::JsonEncodedValueAsString, url_encoded_data::UrlEncodedValueAsString,
    HttpFailResult,
};

pub enum FormDataValueAsString<'s> {
    UrlEncodedValueAsString(&'s UrlEncodedValueAsString<'s>),
    JsonEncodedData(&'s JsonEncodedValueAsString<'s>),
}

impl<'s> FormDataValueAsString<'s> {
    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self {
            FormDataValueAsString::UrlEncodedValueAsString(result) => result.as_string(),

            FormDataValueAsString::JsonEncodedData(result) => result.as_string(),
        }
    }
    pub fn as_bool(&self) -> Result<bool, HttpFailResult> {
        match self {
            FormDataValueAsString::UrlEncodedValueAsString(result) => {
                return result.as_bool();
            }
            FormDataValueAsString::JsonEncodedData(result) => {
                return result.as_bool();
            }
        }
    }
    pub fn parse<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self {
            FormDataValueAsString::UrlEncodedValueAsString(result) => result.parse(),
            FormDataValueAsString::JsonEncodedData(result) => result.parse(),
        }
    }
}
