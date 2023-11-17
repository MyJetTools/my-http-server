use std::str::FromStr;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::de::DeserializeOwned;

use crate::HttpFailResult;

pub fn to_simple_value<T: FromStr>(
    param_name: &str,
    value: &str,
    src: &str,
) -> Result<T, HttpFailResult> {
    if let Ok(result) = value.parse() {
        return Ok(result);
    }
    let err = HttpFailResult::invalid_value_to_parse(format!(
        "Can not parse [{}] from [{}]",
        param_name, src
    ));

    return Err(err);
}

pub fn to_bool(param_name: &str, value: &str, src: &str) -> Result<bool, HttpFailResult> {
    if value == "1" || value.to_lowercase() == "true" {
        return Ok(true);
    }

    if value == "0" || value.to_lowercase() == "false" {
        return Ok(false);
    }

    let err = HttpFailResult::invalid_value_to_parse(format!(
        "Can not parse [{}] as boolean from [{}]",
        param_name, src
    ));

    return Err(err);
}

pub fn to_json<TResult: DeserializeOwned>(
    param_name: &str,
    value: &[u8],
    src: &str,
) -> Result<TResult, HttpFailResult> {
    match serde_json::from_slice(value) {
        Ok(result) => Ok(result),
        Err(_) => Err(HttpFailResult::invalid_value_to_parse(format!(
            "Can not parse {} as json from {}",
            param_name, src
        ))),
    }
}

pub fn to_date_time(
    param_name: &str,
    value: &str,
    src: &str,
) -> Result<DateTimeAsMicroseconds, HttpFailResult> {
    match DateTimeAsMicroseconds::from_str(value) {
        Some(result) => Ok(result),
        None => Err(HttpFailResult::invalid_value_to_parse(format!(
            "Can not parse [{}] as date time  from [{}]",
            param_name, src
        ))),
    }
}
