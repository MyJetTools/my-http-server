use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::HttpFailResult;

pub trait ParseHttpString: Sized {
    fn from_str(s: &str) -> Result<Self, HttpFailResult>;
}

impl ParseHttpString for DateTimeAsMicroseconds {
    fn from_str(s: &str) -> Result<Self, HttpFailResult> {
        match Self::from_str(s) {
            Some(result) => Ok(result),
            None => Err(HttpFailResult::invalid_value_to_parse(format!(
                "Can not parse to date time from string: {}",
                s
            ))),
        }
    }
}
