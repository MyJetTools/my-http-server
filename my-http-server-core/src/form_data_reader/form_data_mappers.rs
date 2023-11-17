use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::de::DeserializeOwned;

use crate::{
    types::{FileContent, RawData, RawDataTyped},
    FormDataItem, HttpFailResult,
};

const SRC_FORM_DATA: &str = "FormData";

impl TryInto<String> for FormDataItem<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name: _ } => Ok(value.to_string()),
            FormDataItem::File {
                name: _,
                file_name: _,
                content_type: _,
                content: _,
            } => Err(HttpFailResult::as_not_supported_content_type(format!(
                "Field contains a File which is not possible to convert to string",
            ))),
        }
    }
}

impl<'s> TryInto<&'s str> for FormDataItem<'s> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<&'s str, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name: _ } => Ok(value),
            FormDataItem::File {
                name,
                file_name: _,
                content_type: _,
                content: _,
            } => Err(HttpFailResult::as_not_supported_content_type(format!(
                "Field {} contains a File which is not possible to convert to string",
                name,
            ))),
        }
    }
}

impl TryInto<DateTimeAsMicroseconds> for FormDataItem<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<DateTimeAsMicroseconds, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name } => {
                crate::convert_from_str::to_date_time(name, value, SRC_FORM_DATA)
            }
            FormDataItem::File {
                name: _,
                file_name: _,
                content_type: _,
                content: _,
            } => Err(HttpFailResult::as_not_supported_content_type(format!(
                "Field contains a File which is not possible to convert to string",
            ))),
        }
    }
}

impl<'s, T: DeserializeOwned> TryInto<RawDataTyped<'s, T>> for FormDataItem<'s> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawDataTyped<'s, T>, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name } => Ok(RawDataTyped::new(
                name.into(),
                value.as_bytes().to_vec(),
                SRC_FORM_DATA,
            )),
            FormDataItem::File {
                name,
                file_name: _,
                content_type: _,
                content,
            } => Ok(RawDataTyped::new(
                name.into(),
                content.to_vec(),
                SRC_FORM_DATA,
            )),
        }
    }
}

impl TryInto<RawData> for FormDataItem<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawData, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name: _ } => {
                Ok(RawData::new(value.as_bytes().to_vec()))
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

impl TryInto<FileContent> for FormDataItem<'_> {
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
