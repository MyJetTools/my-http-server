use std::collections::HashMap;

use crate::{url_decoder::UrlDecodeError, HttpFailResult};

use super::QueryStringValue;

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

pub struct QueryString<'s> {
    query_string: HashMap<String, QueryStringValue<'s>>,
    data_source: QueryStringDataSource,
}

impl<'s> QueryString<'s> {
    pub fn new(src: &'s str, data_source: QueryStringDataSource) -> Result<Self, UrlDecodeError> {
        let result = Self {
            query_string: super::url_utils::parse_query_string(src)?,
            data_source,
        };

        Ok(result)
    }

    pub fn get_required<'r, 't>(
        &'r self,
        name: &'t str,
    ) -> Result<&QueryStringValue<'r>, HttpFailResult> {
        let result = self.query_string.get(name);

        match result {
            Some(e) => Ok(e),
            None => Err(HttpFailResult::required_parameter_is_missing(
                name,
                self.data_source.as_str(),
            )),
        }
    }

    pub fn get_optional<'r, 't>(&'r self, name: &'t str) -> Option<&QueryStringValue<'r>> {
        self.query_string.get(name)
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
    }
}
