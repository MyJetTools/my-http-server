use std::str::FromStr;

use my_json::json_reader::JsonValue;
use rust_extensions::{date_time::DateTimeAsMicroseconds, sorted_vec::EntityWithStrKey};

use crate::HttpFailResult;
use my_json::json_reader::JsonKeyValue;
pub struct JsonEncodedValueAsString<'s> {
    value: JsonValue,
    raw: &'s [u8],
    name: String,
}

impl<'s> EntityWithStrKey for JsonEncodedValueAsString<'s> {
    fn get_key(&self) -> &str {
        self.name.as_str()
    }
}

impl<'s> JsonEncodedValueAsString<'s> {
    pub fn new(json_key_value: JsonKeyValue, raw: &'s [u8]) -> Self {
        let name = json_key_value.name.as_str(&raw).unwrap().to_string();
        Self {
            value: json_key_value.value,
            raw,
            name,
        }
    }

    pub fn as_raw_str(&'s self) -> Result<&'s str, HttpFailResult> {
        match self.value.as_raw_str(&self.raw) {
            Some(result) => Ok(result),
            None => Err(HttpFailResult::required_parameter_is_missing(
                &self.name,
                "body json",
            )),
        }
    }

    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self.value.as_str(&self.raw) {
            Some(result) => Ok(result.to_string()),
            None => Err(HttpFailResult::required_parameter_is_missing(
                &self.name,
                "body json",
            )),
        }
    }
    pub fn as_bool(&self) -> Result<bool, HttpFailResult> {
        match self.value.as_raw_str(&self.raw) {
            Some(result) => crate::convert_from_str::to_bool(&self.name, result, "body json"),
            None => Err(HttpFailResult::required_parameter_is_missing(
                &self.name,
                "body json",
            )),
        }
    }

    pub fn as_date_time(&self) -> Result<DateTimeAsMicroseconds, HttpFailResult> {
        match self.value.as_unescaped_str(&self.raw) {
            Some(result) => crate::convert_from_str::to_date_time(&self.name, result, "body json"),
            None => Err(HttpFailResult::required_parameter_is_missing(
                &self.name,
                "body json",
            )),
        }
    }
    pub fn parse<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self.value.as_str(&self.raw) {
            Some(value) => {
                crate::convert_from_str::to_simple_value(&self.name, value.as_str(), "body json")
            }
            None => Err(HttpFailResult::required_parameter_is_missing(
                &self.name,
                "body json",
            )),
        }
    }

    pub fn as_bytes(&self) -> Result<&[u8], HttpFailResult> {
        let result = self.value.as_bytes(&self.raw);
        Ok(result)
    }
}
