use std::collections::HashMap;

use crate::{url_decoder::UrlDecodeError, HttpFailResult};

use super::UrlEncodedValueAsString;

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

pub struct UrlEncodedData<'s> {
    query_string: HashMap<String, UrlEncodedValueAsString<'s>>,
    data_source: UrlEncodedDataSource,
}

impl<'s> UrlEncodedData<'s> {
    pub fn new(src: &'s str, data_source: UrlEncodedDataSource) -> Result<Self, UrlDecodeError> {
        let result = Self {
            query_string: super::url_utils::parse_query_string(src)?,
            data_source,
        };

        Ok(result)
    }

    pub fn get_required(
        &'s self,
        name: &str,
    ) -> Result<&'s UrlEncodedValueAsString<'s>, HttpFailResult> {
        let result = self.query_string.get(name);

        match result {
            Some(e) => Ok(e),
            None => Err(HttpFailResult::required_parameter_is_missing(
                name,
                self.data_source.as_str(),
            )),
        }
    }

    pub fn get_optional(&'s self, name: &str) -> Option<&'s UrlEncodedValueAsString<'s>> {
        self.query_string.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_basic() {
        let query_string =
            "tableName=deposit-restrictions&partitionKey=%2A&rowKey=1abfc&field=1a+bfc";

        let query_string =
            UrlEncodedData::new(query_string, UrlEncodedDataSource::QueryString).unwrap();

        let result = query_string
            .get_optional("partitionKey")
            .unwrap()
            .as_string()
            .unwrap();

        assert_eq!("*", result);

        let result = query_string
            .get_optional("rowKey")
            .unwrap()
            .as_string()
            .unwrap();

        assert_eq!("1abfc", result);

        let result = query_string
            .get_optional("field")
            .unwrap()
            .as_string()
            .unwrap();

        assert_eq!("1a bfc", result);
    }
}
