use std::collections::HashMap;

use hyper::HeaderMap;

use crate::{data_src::SRC_HEADER, ContentEncoding, HeaderValue, HttpFailResult};

pub trait HttpRequestHeaders {
    fn try_get_case_sensitive<'s>(&'s self, header_name: &'static str) -> Option<HeaderValue<'s>>;
    fn try_get_case_sensitive_as_str(
        &self,
        header_name: &str,
    ) -> Result<Option<&str>, HttpFailResult>;
    fn try_get_case_insensitive_as_str(
        &self,
        header_name: &str,
    ) -> Result<Option<&str>, HttpFailResult>;
    fn try_get_case_insensitive<'s>(&'s self, header_name: &'static str)
        -> Option<HeaderValue<'s>>;

    fn get_required_case_sensitive<'s>(
        &'s self,
        header_name: &'static str,
    ) -> Result<HeaderValue<'s>, HttpFailResult> {
        match self.try_get_case_sensitive(header_name) {
            Some(value) => Ok(value),
            None => Err(HttpFailResult::required_parameter_is_missing(
                header_name,
                SRC_HEADER,
            )),
        }
    }

    fn get_required_case_insensitive<'s>(
        &'s self,
        header_name: &'static str,
    ) -> Result<HeaderValue<'s>, HttpFailResult> {
        match self.try_get_case_insensitive(header_name) {
            Some(value) => Ok(value),
            None => Err(HttpFailResult::required_parameter_is_missing(
                header_name,
                SRC_HEADER,
            )),
        }
    }

    fn to_hash_map(&self) -> HashMap<String, String>;

    fn get_content_encoding(&self) -> Result<ContentEncoding, HttpFailResult> {
        let content_encoding = self.try_get_case_insensitive_as_str("content-encoding")?;

        let result = ContentEncoding::new(content_encoding)?;

        Ok(result)
    }
}

impl HttpRequestHeaders for HeaderMap {
    fn try_get_case_sensitive<'s>(&'s self, header_name: &'static str) -> Option<HeaderValue<'s>> {
        let result = self.get(header_name)?;

        Some(HeaderValue::from_header_value(header_name, result))
    }

    fn try_get_case_insensitive_as_str(
        &self,
        header_to_find: &str,
    ) -> Result<Option<&str>, HttpFailResult> {
        if let Some(value) = self.get(header_to_find) {
            let value = header_value_to_string(header_to_find, value)?;
            return Ok(Some(value));
        }

        for (header_name, header_value) in self.iter() {
            if header_name.as_str().eq_ignore_ascii_case(header_to_find) {
                let value = header_value_to_string(header_to_find, header_value)?;
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    fn try_get_case_sensitive_as_str(
        &self,
        header_name: &str,
    ) -> Result<Option<&str>, HttpFailResult> {
        let result = self.get(header_name);
        if result.is_none() {
            return Ok(None);
        }

        let result = result.unwrap();

        let value = header_value_to_string(header_name, result)?;
        Ok(Some(value))
    }

    fn try_get_case_insensitive<'s>(
        &'s self,
        header_name: &'static str,
    ) -> Option<HeaderValue<'s>> {
        for (key, value) in self {
            if rust_extensions::str_utils::compare_strings_case_insensitive(
                key.as_str(),
                header_name,
            ) {
                return Some(HeaderValue::from_header_value(header_name, value));
            }
        }

        None
    }

    fn to_hash_map(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        for (key, value) in self {
            let key = key.as_str().to_string();
            let value = value.to_str().unwrap().to_string();

            result.insert(key, value);
        }

        result
    }
}

fn header_value_to_string<'s>(
    header_name: &str,
    header_value: &'s hyper::http::HeaderValue,
) -> Result<&'s str, HttpFailResult> {
    match header_value.to_str() {
        Ok(value) => return Ok(value),
        Err(_) => {
            return Err(HttpFailResult::as_validation_error(format!(
                "Header value is not a valid string. Header name: {}",
                header_name
            )));
        }
    }
}
