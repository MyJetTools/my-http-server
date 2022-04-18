use std::collections::HashMap;

use crate::{url_decoder::UrlDecodeError, HttpFailResult};

pub enum QueryStringDataSource {
    Headers,
    FormData,
    QueryString,
}
impl QueryStringDataSource {
    pub fn as_str(&self) -> &str {
        match self {
            QueryStringDataSource::Headers => "headers",
            QueryStringDataSource::FormData => "from data",
            QueryStringDataSource::QueryString => "query string",
        }
    }
}

pub struct QueryString {
    query_string: HashMap<String, String>,
    data_source: QueryStringDataSource,
}

impl QueryString {
    pub fn new(src: &str, data_source: QueryStringDataSource) -> Result<Self, UrlDecodeError> {
        let result = Self {
            query_string: super::url_utils::parse_query_string(src)?,
            data_source,
        };

        Ok(result)
    }

    pub fn get_required<'r, 't>(&'r self, name: &'t str) -> Result<&str, HttpFailResult> {
        let result = self.query_string.get(name);

        match result {
            Some(e) => Ok(e),
            None => Err(HttpFailResult::required_parameter_is_missing(
                name,
                self.data_source.as_str(),
            )),
        }
    }

    pub fn get_optional(&self, name: &str) -> Option<&str> {
        let result = self.query_string.get(name)?;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_basic() {
        let query_string = "tableName=deposit-restrictions&partitionKey=%2A&rowKey=1abfc";

        let query_string =
            QueryString::new(query_string, QueryStringDataSource::QueryString).unwrap();

        let result = query_string.get_optional("partitionKey").unwrap();

        assert_eq!("*", result);

        let result = query_string.get_optional("rowKey").unwrap();

        assert_eq!("1abfc", result);
    }
}
