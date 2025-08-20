use my_json::json_reader::{JsonFirstLineReaderInner, JsonParseError};
use rust_extensions::{array_of_bytes_iterator::SliceIterator, sorted_vec::SortedVecWithStrKey};

use crate::HttpFailResult;

use super::JsonEncodedValueAsString;

pub struct JsonEncodedData<'s> {
    values: SortedVecWithStrKey<JsonEncodedValueAsString<'s>>,
}

impl<'s> JsonEncodedData<'s> {
    pub fn new(raw: &'s [u8]) -> Result<Self, JsonParseError> {
        let mut result = Self {
            values: SortedVecWithStrKey::new(),
        };

        let json_first_line_reader: JsonFirstLineReaderInner<SliceIterator> = raw.into();

        while let Some(line) = json_first_line_reader.get_next() {
            let line = line?;

            if !line.value.is_null(&json_first_line_reader) {
                result
                    .values
                    .insert_or_replace(JsonEncodedValueAsString::new(line, &raw));
            }
        }

        Ok(result)
    }

    pub fn get_required(
        &'s self,
        name: &str,
    ) -> Result<&'s JsonEncodedValueAsString<'s>, HttpFailResult> {
        let result = self.values.get(name);

        match result {
            Some(result) => Ok(result),
            None => Err(HttpFailResult::required_parameter_is_missing(name, "Body")),
        }
    }

    pub fn get_optional(&'s self, name: &str) -> Option<&'s JsonEncodedValueAsString<'s>> {
        self.values.get(name)
    }
}
