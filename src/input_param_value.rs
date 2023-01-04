use std::str::FromStr;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::de::DeserializeOwned;
use url_utils::url_encoded_data_reader::UrlEncodedValueAsString;

use crate::{json_encoded_data::JsonEncodedValueAsString, types::FileContent, HttpFailResult};

pub enum InputParamValue<'s> {
    UrlEncodedValueAsStringRef {
        value: &'s UrlEncodedValueAsString<'s>,
        src: &'static str,
    },
    UrlEncodedValueAsString {
        value: UrlEncodedValueAsString<'s>,
        src: &'static str,
    },
    JsonEncodedData {
        value: &'s JsonEncodedValueAsString<'s>,
        src: &'static str,
    },
    Raw {
        value: &'s str,
        src: &'static str,
    },
    File {
        file: FileContent,
        src: &'static str,
    },
}

impl<'s> InputParamValue<'s> {
    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self {
            InputParamValue::UrlEncodedValueAsStringRef { value, src } => {
                let result = value.as_string();
                return crate::url_encoded_data::convert_error(result, src);
            }

            InputParamValue::UrlEncodedValueAsString { value, src } => {
                let result = value.as_string();
                return crate::url_encoded_data::convert_error(result, src);
            }

            InputParamValue::JsonEncodedData { value, src: _ } => value.as_string(),
            InputParamValue::Raw { value, src: _ } => Ok(value.to_string()),
            InputParamValue::File { src, .. } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "request {} is for a value, but request contains a file",
                    src
                )))
            }
        }
    }

    pub fn from_str<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self {
            InputParamValue::UrlEncodedValueAsString { value, src } => {
                let result = value.parse();
                return crate::url_encoded_data::convert_error(result, src);
            }
            InputParamValue::UrlEncodedValueAsStringRef { value, src } => {
                let result = value.parse();
                return crate::url_encoded_data::convert_error(result, src);
            }
            InputParamValue::JsonEncodedData { value, src: _ } => value.parse(),
            InputParamValue::Raw { value, src } => parse_into_type(value, src),
            InputParamValue::File { src, .. } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "request  is for a value, but request contains a file in {}",
                    src
                )))
            }
        }
    }

    pub fn get_raw_str(&self) -> Result<&str, HttpFailResult> {
        match self {
            InputParamValue::UrlEncodedValueAsStringRef { value, src: _ } => Ok(value.value),
            InputParamValue::UrlEncodedValueAsString { value, src: _ } => Ok(value.value),
            InputParamValue::JsonEncodedData { value, src: _ } => value.as_raw_str(),
            InputParamValue::Raw { value, src: _ } => Ok(value),
            InputParamValue::File { src, .. } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "request is for a get_raw_str, but request contains a file in {}",
                    src
                )))
            }
        }
    }

    pub fn from_json<TResult: DeserializeOwned>(&self) -> Result<TResult, HttpFailResult> {
        match self {
            InputParamValue::UrlEncodedValueAsStringRef { value, src: _ } => {
                parse_json_value(value.as_string()?.as_str())
            }
            InputParamValue::UrlEncodedValueAsString { value, src: _ } => {
                parse_json_value(value.as_string()?.as_str())
            }
            InputParamValue::JsonEncodedData { value, src: _ } => {
                parse_json_value(value.as_string()?.as_str())
            }
            InputParamValue::Raw { value, src: _ } => parse_json_value(value),
            InputParamValue::File { src, .. } => parse_json_value(src),
        }
    }

    fn get_src(&self) -> &str {
        match self {
            InputParamValue::UrlEncodedValueAsStringRef { src, .. } => src,
            InputParamValue::UrlEncodedValueAsString { src, .. } => src,
            InputParamValue::JsonEncodedData { src, .. } => src,
            InputParamValue::Raw { src, .. } => src,
            InputParamValue::File { src, .. } => src,
        }
    }
}

pub fn parse_date_time(value: &str, src: &str) -> Result<DateTimeAsMicroseconds, HttpFailResult> {
    match DateTimeAsMicroseconds::from_str(value) {
        Some(result) => Ok(result),
        None => Err(HttpFailResult::invalid_value_to_parse(format!(
            "Can not parse [{}] as date time  from [{}]",
            value, src
        ))),
    }
}

pub fn parse_bool_value(value: &str, src: &str) -> Result<bool, HttpFailResult> {
    if value == "1" || value.to_lowercase() == "true" {
        return Ok(true);
    }

    if value == "0" || value.to_lowercase() == "false" {
        return Ok(false);
    }

    let err = HttpFailResult::invalid_value_to_parse(format!(
        "Can not parse [{}] as boolean from [{}]",
        value, src
    ));

    return Err(err);
}

pub fn parse_into_type<T: FromStr>(value: &str, src: &str) -> Result<T, HttpFailResult> {
    let result = T::from_str(value);
    return match result {
        Ok(value) => Ok(value),
        _ => Err(HttpFailResult::invalid_value_to_parse(format!(
            "Can not parse [{}] value  from [{}]",
            value, src
        ))),
    };
}

impl TryInto<DateTimeAsMicroseconds> for InputParamValue<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<DateTimeAsMicroseconds, Self::Error> {
        match self {
            InputParamValue::UrlEncodedValueAsStringRef { value, src } => {
                let result = value.as_date_time();
                return crate::url_encoded_data::convert_error(result, src);
            }
            InputParamValue::UrlEncodedValueAsString { value, src } => {
                let result = value.as_date_time();
                return crate::url_encoded_data::convert_error(result, src);
            }
            InputParamValue::JsonEncodedData { value, src: _ } => {
                return value.as_date_time();
            }
            InputParamValue::Raw { value, src } => parse_date_time(value, src),
            InputParamValue::File { src, .. } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "reading DateTime, but request contains a file in {}",
                    src
                )))
            }
        }
    }
}

pub fn parse_json_value<TResult: DeserializeOwned>(src: &str) -> Result<TResult, HttpFailResult> {
    match serde_json::from_str(src) {
        Ok(result) => Ok(result),
        Err(_) => Err(HttpFailResult::invalid_value_to_parse(format!(
            "Can not parse [{}] as json",
            src
        ))),
    }
}

impl<'s> From<&'s UrlEncodedValueAsString<'s>> for InputParamValue<'s> {
    fn from(src: &'s UrlEncodedValueAsString) -> Self {
        InputParamValue::UrlEncodedValueAsStringRef {
            value: src,
            src: "query",
        }
    }
}

impl TryInto<bool> for InputParamValue<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<bool, Self::Error> {
        return parse_bool_value(self.get_raw_str()?, self.get_src());
    }
}

impl TryInto<u8> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u8, Self::Error> {
        self.from_str()
    }
}

impl TryInto<i8> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i8, Self::Error> {
        self.from_str()
    }
}

impl TryInto<u16> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u16, Self::Error> {
        self.from_str()
    }
}

impl TryInto<i16> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i16, Self::Error> {
        self.from_str()
    }
}

impl TryInto<u32> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u32, Self::Error> {
        self.from_str()
    }
}

impl TryInto<i32> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i32, Self::Error> {
        self.from_str()
    }
}

impl TryInto<u64> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u64, Self::Error> {
        self.from_str()
    }
}

impl TryInto<i64> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i64, Self::Error> {
        self.from_str()
    }
}

impl TryInto<usize> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<usize, Self::Error> {
        self.from_str()
    }
}

impl TryInto<isize> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<isize, Self::Error> {
        self.from_str()
    }
}

impl TryInto<f64> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f64, Self::Error> {
        self.from_str()
    }
}

impl TryInto<f32> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f32, Self::Error> {
        self.from_str()
    }
}

impl TryInto<String> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<String, Self::Error> {
        self.as_string()
    }
}
