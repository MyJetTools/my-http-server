use url_utils::{
    url_decoder::UrlDecodeError,
    url_encoded_data_reader::{UrlEncodedDataReader, UrlEncodedValueAsString},
};

use crate::HttpFailResult;

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
    Headers(UrlEncodedDataInner<'s>),
    FormData(UrlEncodedDataInner<'s>),
    QueryString(UrlEncodedDataInner<'s>),
    QueryStringEmpty,
}

pub struct UrlEncodedDataInner<'s> {
    data_reader: UrlEncodedDataReader<'s>,
    data_source: UrlEncodedDataSource,
}

impl<'s> UrlEncodedDataInner<'s> {
    pub fn new(src: &'s str, data_source: UrlEncodedDataSource) -> Result<Self, UrlDecodeError> {
        let result = Self {
            data_reader: UrlEncodedDataReader::new(src)?,
            data_source,
        };

        Ok(result)
    }

    pub fn get_required(
        &'s self,
        name: &str,
    ) -> Result<&'s UrlEncodedValueAsString<'s>, HttpFailResult> {
        let result = self.data_reader.get_required(name);
        return super::convert_error(result, self.data_source);
    }

    pub fn get_optional(&'s self, name: &str) -> Option<&'s UrlEncodedValueAsString<'s>> {
        return self.data_reader.get_optional(name);
    }
}
