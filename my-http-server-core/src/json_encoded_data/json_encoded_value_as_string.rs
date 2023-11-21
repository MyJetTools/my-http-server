use std::str::FromStr;

use my_json::json_reader::JsonValue;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::HttpFailResult;

pub struct JsonEncodedValueAsString<'s> {
    name: &'s str,
    json_value: JsonValue<'s>,
}

impl<'s> JsonEncodedValueAsString<'s> {
    pub fn new(name: &'s str, json_value: JsonValue<'s>) -> Self {
        Self { name, json_value }
    }

    pub fn as_raw_str(&'s self) -> Result<&'s str, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => Ok(result),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name,
                "body json",
            )),
        }
    }

    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => Ok(result.to_string()),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name,
                "body json",
            )),
        }
    }
    pub fn as_bool(&self) -> Result<bool, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => crate::convert_from_str::to_bool(self.name, result, "body json"),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name,
                "body json",
            )),
        }
    }

    pub fn as_date_time(&self) -> Result<DateTimeAsMicroseconds, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => crate::convert_from_str::to_date_time(self.name, result, "body json"),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name,
                "body json",
            )),
        }
    }
    pub fn parse<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self.json_value.as_str() {
            Some(value) => crate::convert_from_str::to_simple_value(self.name, value, "body json"),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name,
                "body json",
            )),
        }
    }

    pub fn as_bytes(&self) -> Result<&[u8], HttpFailResult> {
        match self.json_value.as_bytes() {
            Some(value) => Ok(value),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name,
                "body json",
            )),
        }
    }
}
