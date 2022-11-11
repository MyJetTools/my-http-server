use std::str::FromStr;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use url_utils::url_encoded_data_reader::UrlEncodedValueAsString;

use crate::{json_encoded_data::JsonEncodedValueAsString, HttpFailResult};

pub enum ValueAsString<'s> {
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
}

impl<'s> ValueAsString<'s> {
    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self {
            ValueAsString::UrlEncodedValueAsStringRef { value, src } => {
                let result = value.as_string();
                return crate::url_encoded_data::convert_error(result, src);
            }

            ValueAsString::UrlEncodedValueAsString { value, src } => {
                let result = value.as_string();
                return crate::url_encoded_data::convert_error(result, src);
            }

            ValueAsString::JsonEncodedData { value, src: _ } => value.as_string(),
            ValueAsString::Raw { value, src: _ } => Ok(value.to_string()),
        }
    }

    pub fn parse<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self {
            ValueAsString::UrlEncodedValueAsString { value, src } => {
                let result = value.parse();
                return crate::url_encoded_data::convert_error(result, src);
            }
            ValueAsString::UrlEncodedValueAsStringRef { value, src } => {
                let result = value.parse();
                return crate::url_encoded_data::convert_error(result, src);
            }
            ValueAsString::JsonEncodedData { value, src: _ } => value.parse(),
            ValueAsString::Raw { value, src } => parse_into_type(value, src),
        }
    }

    pub fn get_raw_str(&self) -> Result<&str, HttpFailResult> {
        match self {
            ValueAsString::UrlEncodedValueAsStringRef { value, src: _ } => Ok(value.value),
            ValueAsString::UrlEncodedValueAsString { value, src: _ } => Ok(value.value),
            ValueAsString::JsonEncodedData { value, src: _ } => value.as_raw_str(),
            ValueAsString::Raw { value, src: _ } => Ok(value),
        }
    }

    fn get_src(&self) -> &str {
        match self {
            ValueAsString::UrlEncodedValueAsStringRef { value: _, src } => src,
            ValueAsString::UrlEncodedValueAsString { value: _, src } => src,
            ValueAsString::JsonEncodedData { value: _, src } => src,
            ValueAsString::Raw { value: _, src } => src,
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
    let result = value.parse::<T>();
    return match result {
        Ok(value) => Ok(value),
        _ => Err(HttpFailResult::invalid_value_to_parse(format!(
            "Can not parse [{}] value  from [{}]",
            value, src
        ))),
    };
}

impl TryInto<DateTimeAsMicroseconds> for ValueAsString<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<DateTimeAsMicroseconds, Self::Error> {
        match self {
            ValueAsString::UrlEncodedValueAsStringRef { value, src } => {
                let result = value.as_date_time();
                return crate::url_encoded_data::convert_error(result, src);
            }
            ValueAsString::UrlEncodedValueAsString { value, src } => {
                let result = value.as_date_time();
                return crate::url_encoded_data::convert_error(result, src);
            }
            ValueAsString::JsonEncodedData { value, src: _ } => {
                return value.as_date_time();
            }
            ValueAsString::Raw { value, src } => parse_date_time(value, src),
        }
    }
}

impl<'s> From<&'s UrlEncodedValueAsString<'s>> for ValueAsString<'s> {
    fn from(src: &'s UrlEncodedValueAsString) -> Self {
        ValueAsString::UrlEncodedValueAsStringRef {
            value: src,
            src: "query",
        }
    }
}

impl TryInto<bool> for ValueAsString<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<bool, Self::Error> {
        return parse_bool_value(self.get_raw_str()?, self.get_src());
    }
}

impl TryInto<u8> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u8, Self::Error> {
        self.parse()
    }
}

impl TryInto<i8> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i8, Self::Error> {
        self.parse()
    }
}

impl TryInto<u16> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u16, Self::Error> {
        self.parse()
    }
}

impl TryInto<i16> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i16, Self::Error> {
        self.parse()
    }
}

impl TryInto<u32> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u32, Self::Error> {
        self.parse()
    }
}

impl TryInto<i32> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i32, Self::Error> {
        self.parse()
    }
}

impl TryInto<u64> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u64, Self::Error> {
        self.parse()
    }
}

impl TryInto<i64> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i64, Self::Error> {
        self.parse()
    }
}

impl TryInto<String> for ValueAsString<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<String, Self::Error> {
        self.as_string()
    }
}
