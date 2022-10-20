use url_utils::{
    url_decoder::UrlDecodeError,
    url_encoded_data_reader::{UrlEncodedDataReader, UrlEncodedValueAsString},
};

use crate::{form_data::FORM_DATA_SRC, HttpFailResult};

#[derive(Debug, Clone, Copy)]
pub enum UrlEncodedDataSource {
    Headers,
    FormData,
    QueryString,
}
impl UrlEncodedDataSource {
    pub fn as_str(&self) -> &str {
        match self {
            UrlEncodedDataSource::Headers => "headers",
            UrlEncodedDataSource::FormData => "from data",
            UrlEncodedDataSource::QueryString => "query string",
        }
    }
}

pub enum UrlEncodedData<'s> {
    Headers(UrlEncodedDataReader<'s>),
    FormData(UrlEncodedDataReader<'s>),
    QueryString(UrlEncodedDataReader<'s>),
    QueryStringEmpty,
}

impl<'s> UrlEncodedData<'s> {
    pub fn from_headers(src: &'s str) -> Result<Self, UrlDecodeError> {
        let result = UrlEncodedDataReader::new(src)?;
        Ok(Self::Headers(result))
    }

    pub fn from_form_data(src: &'s str) -> Result<Self, UrlDecodeError> {
        let result = UrlEncodedDataReader::new(src)?;
        Ok(Self::FormData(result))
    }

    pub fn from_query_string(src: &'s str) -> Result<Self, UrlDecodeError> {
        let result = UrlEncodedDataReader::new(src)?;
        Ok(Self::QueryString(result))
    }

    pub fn new_query_string_empty() -> Self {
        Self::QueryStringEmpty
    }

    pub fn get_required(
        &'s self,
        name: &str,
    ) -> Result<&'s UrlEncodedValueAsString<'s>, HttpFailResult> {
        match self {
            UrlEncodedData::Headers(src) => {
                let result = src.get_required(name);
                return super::convert_error(result, self.get_source_as_string());
            }
            UrlEncodedData::FormData(src) => {
                let result = src.get_required(name);
                return super::convert_error(result, self.get_source_as_string());
            }
            UrlEncodedData::QueryString(src) => {
                let result = src.get_required(name);
                return super::convert_error(result, self.get_source_as_string());
            }
            UrlEncodedData::QueryStringEmpty => Err(HttpFailResult::required_parameter_is_missing(
                name,
                self.get_source_as_string(),
            )),
        }
    }

    pub fn get_optional(&'s self, name: &str) -> Option<&'s UrlEncodedValueAsString<'s>> {
        match self {
            UrlEncodedData::Headers(src) => src.get_optional(name),
            UrlEncodedData::FormData(src) => src.get_optional(name),
            UrlEncodedData::QueryString(src) => src.get_optional(name),
            UrlEncodedData::QueryStringEmpty => None,
        }
    }

    pub fn get_source_as_string(&self) -> &'static str {
        match self {
            UrlEncodedData::Headers(_) => "headers",
            UrlEncodedData::FormData(_) => FORM_DATA_SRC,
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
