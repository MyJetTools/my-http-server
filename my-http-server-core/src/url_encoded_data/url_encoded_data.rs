use url_utils::{url_decoder::UrlDecodeError, url_encoded_data_reader::UrlEncodedDataReader};

use crate::HttpFailResult;
use url_utils::url_encoded_data_reader::UrlEncodedValue;

pub enum UrlEncodedData<'s> {
    Body(UrlEncodedDataReader<'s>),
    QueryString(UrlEncodedDataReader<'s>),
    QueryStringEmpty,
}

impl<'s> UrlEncodedData<'s> {
    pub fn from_body(src: &'s str) -> Result<Self, UrlDecodeError> {
        let result = UrlEncodedDataReader::new(src)?;
        Ok(Self::Body(result))
    }

    pub fn from_query_string(src: &'s str) -> Result<Self, UrlDecodeError> {
        let result = UrlEncodedDataReader::new(src)?;
        Ok(Self::QueryString(result))
    }

    pub fn new_query_string_empty() -> Self {
        Self::QueryStringEmpty
    }

    pub fn get_required(&'s self, name: &str) -> Result<UrlEncodedValue<'s>, HttpFailResult> {
        match self {
            UrlEncodedData::Body(src) => {
                let result = src.get_required(name);
                return super::convert_error(name, result, self.get_source_as_string());
            }
            UrlEncodedData::QueryString(src) => {
                let result = src.get_required(name);
                return super::convert_error(name, result, self.get_source_as_string());
            }
            UrlEncodedData::QueryStringEmpty => Err(HttpFailResult::required_parameter_is_missing(
                name,
                self.get_source_as_string(),
            )),
        }
    }

    pub fn get_optional(&'s self, name: &str) -> Option<UrlEncodedValue<'s>> {
        match self {
            UrlEncodedData::Body(src) => src.get_optional(name),
            UrlEncodedData::QueryString(src) => src.get_optional(name),
            UrlEncodedData::QueryStringEmpty => None,
        }
    }

    pub fn get_vec(
        &'s self,
        name: &'static str,
    ) -> Result<Vec<UrlEncodedValue<'s>>, HttpFailResult> {
        match self {
            UrlEncodedData::Body(src) => {
                let result = src.get_vec(name);
                return Ok(result);
            }
            UrlEncodedData::QueryString(src) => {
                let result = src.get_vec(name);
                return Ok(result);
            }
            UrlEncodedData::QueryStringEmpty => {
                return Ok(vec![]);
            }
        }
    }

    pub fn get_source_as_string(&self) -> &'static str {
        match self {
            UrlEncodedData::Body(_) => "body",
            UrlEncodedData::QueryString(_) => "query string",
            UrlEncodedData::QueryStringEmpty => "query string",
        }
    }
}

impl From<UrlDecodeError> for HttpFailResult {
    fn from(src: UrlDecodeError) -> Self {
        Self::as_fatal_error(format!("{:?}", src))
    }
}
