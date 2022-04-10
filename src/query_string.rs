use std::{collections::HashMap, str::FromStr};

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

    pub fn get_required_string_parameter<'r, 't>(
        &'r self,
        name: &'t str,
    ) -> Result<&'r str, HttpFailResult> {
        let result = self.query_string.get(name);

        match result {
            Some(e) => Ok(e),
            None => Err(HttpFailResult::required_parameter_is_missing(
                name,
                self.data_source.as_str(),
            )),
        }
    }

    pub fn get_optional_string_parameter<'r, 't>(&'r self, name: &'t str) -> Option<&'r str> {
        let result = self.query_string.get(name)?;
        Some(result.as_str())
    }

    pub fn get_optional_bool_parameter<'r, 't>(
        &'r self,
        name: &'t str,
    ) -> Result<Option<bool>, HttpFailResult> {
        let result = self.query_string.get(name);

        match result {
            Some(value) => {
                let bool_value = parse_bool_value(value)?;
                Ok(Some(bool_value))
            }
            None => Ok(None),
        }
    }

    pub fn get_required_bool_parameter<'r, 't>(
        &'r self,
        name: &'t str,
    ) -> Result<bool, HttpFailResult> {
        let result = self.query_string.get(name);

        match result {
            Some(value) => {
                let bool_value = parse_bool_value(value)?;
                Ok(bool_value)
            }
            None => {
                return HttpFailResult::required_parameter_is_missing(
                    name,
                    self.data_source.as_str(),
                )
                .into_err()
            }
        }
    }

    pub fn get_optional_parameter<'r, 't, T: FromStr>(&'r self, name: &'t str) -> Option<T> {
        let result = self.query_string.get(name);

        match result {
            Some(value) => {
                let result = value.parse::<T>();

                return match result {
                    Ok(value) => Some(value),
                    _ => None,
                };
            }
            None => return None,
        };
    }

    pub fn get_required_parameter<'r, 't, T: FromStr>(
        &'r self,
        name: &'t str,
    ) -> Result<T, HttpFailResult> {
        let result = self.query_string.get(name);

        match result {
            Some(value) => {
                let result = value.parse::<T>();

                return match result {
                    Ok(value) => Ok(value),
                    _ => Err(HttpFailResult::required_parameter_is_missing(
                        name,
                        self.data_source.as_str(),
                    )),
                };
            }
            None => {
                return Err(HttpFailResult::required_parameter_is_missing(
                    name,
                    self.data_source.as_str(),
                ))
            }
        };
    }
}

fn parse_bool_value(value: &str) -> Result<bool, HttpFailResult> {
    let value = value.to_lowercase();
    if value == "1" || value.to_lowercase() == "true" {
        return Ok(true);
    }

    if value == "0" || value.to_lowercase() == "false" {
        return Ok(false);
    }

    let err =
        HttpFailResult::invalid_value_to_parse(format!("Can not parse [{}] as boolean", value));

    return Err(err);
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
            .get_optional_string_parameter("partitionKey")
            .unwrap();

        assert_eq!("*", result);

        let result = query_string
            .get_optional_string_parameter("rowKey")
            .unwrap();
        assert_eq!("1abfc", result);
    }
}
