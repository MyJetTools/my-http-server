use serde::de::DeserializeOwned;
use url_utils::server::{FormDataItem, ReadingFromDataError};

use crate::{data_src::*, types::*, HttpFailResult};

impl<'s, T: DeserializeOwned> TryInto<RawDataTyped<T>> for FormDataItem<'s> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawDataTyped<T>, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, .. } => {
                return Ok(RawDataTyped::new(value.as_bytes().to_vec(), SRC_FORM_DATA));
            }

            FormDataItem::File {
                name: _,
                file_name: _,
                content_type: _,
                content,
            } => Ok(RawDataTyped::new(content.to_vec(), SRC_FORM_DATA)),
        }
    }
}

impl<'s> TryInto<RawData> for &'s FormDataItem<'s> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawData, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, .. } => {
                return Ok(RawData::new(value.as_bytes().to_vec()));
            }
            FormDataItem::File {
                name: _,
                file_name: _,
                content_type: _,
                content,
            } => Ok(RawData::new(content.to_vec())),
        }
    }
}

impl<'s> TryInto<FileContent> for &'s FormDataItem<'s> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<FileContent, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value: _, name } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "Field {} contains a value which is not possible to convert to a file",
                    name,
                )))
            }
            FormDataItem::File {
                name: _,
                file_name,
                content_type,
                content,
            } => Ok(FileContent {
                content_type: content_type.to_string(),
                file_name: file_name.to_string(),
                content: content.to_vec(),
            }),
        }
    }
}

impl From<ReadingFromDataError> for HttpFailResult {
    fn from(value: ReadingFromDataError) -> Self {
        match value {
            ReadingFromDataError::ParameterMissing(param_name) => {
                HttpFailResult::required_parameter_is_missing(&param_name, "FORM_DATA")
            }
            ReadingFromDataError::ValidationError { field, error } => {
                HttpFailResult::as_validation_error(format!(
                    "Parameter: {}. Error: {}",
                    field, error
                ))
            }
        }
    }
}
