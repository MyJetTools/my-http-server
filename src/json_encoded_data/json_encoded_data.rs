use std::collections::HashMap;

use my_json::json_reader::{JsonFirstLineReader, JsonParseError};

use crate::HttpFailResult;

use super::JsonEncodedValueAsString;

pub struct JsonEncodedData<'s> {
    values: HashMap<String, JsonEncodedValueAsString<'s>>,
}

impl<'s> JsonEncodedData<'s> {
    pub fn new(raw: &'s [u8]) -> Result<Self, JsonParseError> {
        let mut result = Self {
            values: HashMap::new(),
        };
        for line in JsonFirstLineReader::new(raw) {
            let line = line?;

            let name = line.get_name()?;

            let value = line.get_value()?;

            if !value.is_null() {
                result
                    .values
                    .insert(name.to_string(), JsonEncodedValueAsString::new(name, value));
            }
        }

        Ok(result)
    }

    pub fn get_required(
        &'s self,
        name: &'s str,
    ) -> Result<&'s JsonEncodedValueAsString<'s>, HttpFailResult> {
        let result = self.values.get(name);

        match result {
            Some(result) => Ok(result),
            None => Err(HttpFailResult::required_parameter_is_missing(
                name, "FormData",
            )),
        }
    }

    pub fn get_optional(&'s self, name: &'s str) -> Option<&'s JsonEncodedValueAsString<'s>> {
        self.values.get(name)
    }
}
