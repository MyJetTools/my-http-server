use hyper::HeaderMap;

use crate::{data_src::SRC_HEADER, HeaderValue, HttpFailResult};

pub trait HttpRequestHeaders {
    fn try_get_case_sensitive(&self, header_name: &'static str) -> Option<HeaderValue>;
    fn try_get_case_sensitive_as_str(&self, header_name: &str) -> Option<&str>;
    fn try_get_case_insensitive(&self, header_name: &'static str) -> Option<HeaderValue>;

    fn get_required_case_sensitive(
        &self,
        header_name: &'static str,
    ) -> Result<HeaderValue, HttpFailResult> {
        match self.try_get_case_sensitive(header_name) {
            Some(value) => Ok(value),
            None => Err(HttpFailResult::required_parameter_is_missing(
                header_name,
                SRC_HEADER,
            )),
        }
    }

    fn get_required_case_insensitive(
        &self,
        header_name: &'static str,
    ) -> Result<HeaderValue, HttpFailResult> {
        match self.try_get_case_insensitive(header_name) {
            Some(value) => Ok(value),
            None => Err(HttpFailResult::required_parameter_is_missing(
                header_name,
                SRC_HEADER,
            )),
        }
    }
}

impl HttpRequestHeaders for HeaderMap {
    fn try_get_case_sensitive(&self, header_name: &'static str) -> Option<HeaderValue> {
        let result = self.get(header_name)?;

        Some(HeaderValue::from_header_value(header_name, result))
    }

    fn try_get_case_sensitive_as_str(&self, header_name: &str) -> Option<&str> {
        let result = self.get(header_name)?;
        Some(result.to_str().unwrap())
    }

    fn try_get_case_insensitive(&self, header_name: &'static str) -> Option<HeaderValue> {
        let header_name_lk = header_name.to_lowercase();

        for (key, value) in self {
            if key.as_str().to_lowercase() == header_name_lk {
                return Some(HeaderValue::from_header_value(header_name, value));
            }
        }

        None
    }
}
