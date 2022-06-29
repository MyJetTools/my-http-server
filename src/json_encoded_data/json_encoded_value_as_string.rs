use std::str::FromStr;

use my_json::json_reader::JsonValue;

use crate::HttpFailResult;

pub struct JsonEncodedValueAsString<'s> {
    name: &'s str,
    json_value: JsonValue<'s>,
}

impl<'s> JsonEncodedValueAsString<'s> {
    pub fn new(name: &'s str, json_value: JsonValue<'s>) -> Self {
        Self { name, json_value }
    }

    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => Ok(result.to_string()),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name, "FormData",
            )),
        }
    }
    pub fn as_bool(&self) -> Result<bool, HttpFailResult> {
        match self.json_value.as_str() {
            Some(result) => parse_bool_value(result),
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name, "FormData",
            )),
        }
    }
    pub fn parse<T: FromStr>(&self) -> Result<T, HttpFailResult> {
        match self.json_value.as_str() {
            Some(value) => match value.parse::<T>() {
                Ok(result) => Ok(result),
                Err(_) => Err(HttpFailResult::invalid_value_to_parse(format!(
                    "Can not parse value {:?}",
                    self.json_value.as_str()
                ))),
            },
            None => Err(HttpFailResult::required_parameter_is_missing(
                self.name, "FormData",
            )),
        }
    }
}

fn parse_bool_value(value: &str) -> Result<bool, HttpFailResult> {
    let value = value.to_lowercase();
    if value == "1" || value.to_lowercase() == "true" {
        return Ok(true);
    }

    if value == "0" || value.to_lowercase() == "false" {
        return Ok(false);
    }

    let err =
        HttpFailResult::invalid_value_to_parse(format!("Can not parse [{}] as boolean", value));

    return Err(err);
}
