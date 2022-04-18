use std::str::FromStr;

use crate::HttpFailResult;

pub struct QueryStringValue<'s> {
    value: &'s str,
}

impl<'s> QueryStringValue<'s> {
    pub fn new(value: &'s str) -> Self {
        Self { value }
    }

    pub fn as_string(&self) -> Result<String, HttpFailResult> {
        let result = crate::url_decoder::decode_from_url_query_string(self.value)?;
        Ok(result)
    }

    pub fn as_bool(&'s self) -> Result<bool, HttpFailResult> {
        let bool_value = parse_bool_value(self.value)?;
        Ok(bool_value)
    }

    pub fn parse<'t, T: FromStr>(&'s self) -> Result<T, HttpFailResult> {
        let result = self.value.parse::<T>();
        return match result {
            Ok(value) => Ok(value),
            _ => Err(HttpFailResult::invalid_value_to_parse(format!(
                "Can not parse value {}",
                self.value
            ))),
        };
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
