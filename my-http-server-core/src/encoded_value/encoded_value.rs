use std::str::FromStr;

use rust_extensions::StrOrString;
use serde::de::DeserializeOwned;

use crate::{
    data_src::SRC_FORM_DATA, json_encoded_data::JsonEncodedValueAsString, FormDataItem,
    HttpFailResult,
};

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

    FormData {
        name: &'static str,
        value: &'s FormDataItem<'s>,
    },
}

impl<'s> EncodedParamValue<'s> {
    pub fn from_url_encoded_data(value: UrlEncodedValue<'s>, src: &'static str) -> Self {
        Self::UrlEncodedValue { value, src }
    }
    pub fn get_name(&self) -> &str {
        match self {
            Self::UrlEncodedValue { value, .. } => value.get_name(),
            Self::JsonEncodedData { name, .. } => &name,
            Self::FormData { name, .. } => name,
        }
    }
    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src } => {
                let result = value.as_string();
                return crate::url_encoded_data::convert_error(value.get_name(), result, src);
            }

            Self::JsonEncodedData {
                name: _,
                value,
                src: _,
            } => value.as_string(),
            EncodedParamValue::FormData { name: _, value } => {
                let value = value.unwrap_as_string()?;
                Ok(value.to_string())
            }
        }
    }

    pub fn from_str<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src } => {
                let result = value.parse();
                return crate::url_encoded_data::convert_error(value.get_name(), result, src);
            }
            Self::JsonEncodedData {
                name: _,
                value,
                src: _,
            } => value.parse(),

            EncodedParamValue::FormData { name, value } => {
                let value = value.unwrap_as_string()?;
                crate::convert_from_str::to_simple_value(name, value, SRC_FORM_DATA)
            }
        }
    }

    pub fn get_raw_str(&self) -> Result<&str, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src: _ } => Ok(value.value),
            Self::JsonEncodedData {
                name: _,
                value,
                src: _,
            } => value.as_raw_str(),

            EncodedParamValue::FormData { name: _, value } => {
                let value = value.unwrap_as_string()?;
                Ok(value)
            }
        }
    }

    pub fn from_json<TResult: DeserializeOwned>(&self) -> Result<TResult, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src } => crate::convert_from_str::to_json(
                value.get_name(),
                &Some(value.as_string()?.as_bytes()),
                src,
            ),
            Self::JsonEncodedData { name, value, src } => {
                crate::convert_from_str::to_json(name, &Some(value.as_bytes()?), src)
            }
            Self::FormData { name, value } => {
                let value = value.unwrap_as_string()?;
                crate::convert_from_str::to_json(name, &Some(value.as_bytes()), SRC_FORM_DATA)
            }
        }
    }

    pub fn get_src(&self) -> &str {
        match self {
            Self::UrlEncodedValue { src, .. } => src,
            Self::JsonEncodedData { src, .. } => src,
            Self::FormData { .. } => SRC_FORM_DATA,
        }
    }

    pub fn as_str(&self) -> Result<StrOrString, HttpFailResult> {
        match self {
            Self::UrlEncodedValue { value, src: _ } => {
                Ok(StrOrString::create_as_string(value.as_string()?))
            }
            Self::JsonEncodedData {
                name: _,
                value,
                src: _,
            } => Ok(StrOrString::create_as_str(value.as_raw_str()?)),
            Self::FormData { value, .. } => {
                let value = value.unwrap_as_string()?;
                Ok(StrOrString::create_as_str(value))
            }
        }
    }
}
