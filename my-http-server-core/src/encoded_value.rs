use std::str::FromStr;

use rust_extensions::StrOrString;
use serde::de::DeserializeOwned;

use crate::{json_encoded_data::JsonEncodedValueAsString, HttpFailResult};

use url_utils::url_encoded_data_reader::UrlEncodedValue;

pub enum EncodedParamValue<'s> {
    UrlEncodedValue {
        value: UrlEncodedValue<'s>,
        src: &'static str,
    },
    JsonEncodedData {
        name: &'static str,
        value: &'s JsonEncodedValueAsString<'s>,
        src: &'static str,
    },
}

impl<'s> EncodedParamValue<'s> {
    pub fn get_name(&self) -> &str {
        match self {
            Self::UrlEncodedValue { value, .. } => value.get_name(),
            Self::JsonEncodedData { name, .. } => &name,
        }
    }
    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src } => {
                let result = value.as_string();
                return crate::url_encoded_data::convert_error(value.get_name(), result, src);
            }

            Self::JsonEncodedData {
                name,
                value,
                src: _,
            } => value.as_string(),
        }
    }

    pub fn from_str<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src } => {
                let result = value.parse();
                return crate::url_encoded_data::convert_error(value.get_name(), result, src);
            }
            Self::JsonEncodedData {
                name,
                value,
                src: _,
            } => value.parse(),
        }
    }

    pub fn get_raw_str(&self) -> Result<&str, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src: _ } => Ok(value.value),
            Self::JsonEncodedData {
                name,
                value,
                src: _,
            } => value.as_raw_str(),
        }
    }

    pub fn from_json<TResult: DeserializeOwned>(&self) -> Result<TResult, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src } => crate::convert_from_str::to_json(
                value.get_name(),
                value.as_string()?.as_bytes(),
                src,
            ),
            Self::JsonEncodedData { name, value, src } => {
                crate::convert_from_str::to_json(name, value.as_bytes()?, src)
            }
        }
    }

    pub fn get_src(&self) -> &str {
        match self {
            Self::UrlEncodedValue { src, .. } => src,
            Self::JsonEncodedData { src, .. } => src,
        }
    }

    pub fn as_str(&self) -> Result<StrOrString, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src: _ } => {
                Ok(StrOrString::create_as_string(value.as_string()?))
            }
            Self::JsonEncodedData {
                name,
                value,
                src: _,
            } => Ok(StrOrString::create_as_str(value.as_raw_str()?)),
        }
    }
}
