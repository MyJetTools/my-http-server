use std::str::FromStr;

use my_json::json_reader::JsonValue;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{value_as_string::*, HttpFailResult, BODY_SRC};

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
                self.name, BODY_SRC,
            )),
        }
    }

    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => Ok(result.to_string()),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name, BODY_SRC,
            )),
        }
    }
    pub fn as_bool(&self) -> Result<bool, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => parse_bool_value(result, BODY_SRC),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name, BODY_SRC,
            )),
        }
    }

    pub fn as_date_time(&self) -> Result<DateTimeAsMicroseconds, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => parse_date_time(result, BODY_SRC),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name, BODY_SRC,
            )),
        }
    }
    pub fn parse<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self.json_value.as_str() {
            Some(value) => parse_into_type(value, BODY_SRC),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name, BODY_SRC,
            )),
        }
    }
}
