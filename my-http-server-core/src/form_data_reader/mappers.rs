use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::de::DeserializeOwned;

use crate::{
    data_src::*,
    form_data_reader::FormDataItem,
    types::{FileContent, RawData, RawDataTyped},
    HttpFailResult,
};

impl<'s> TryInto<String> for &'s FormDataItem<'s> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name } => match value {
                Some(v) => Ok(v.to_string()),
                None => Err(HttpFailResult::required_parameter_is_missing(
                    name,
                    SRC_FORM_DATA,
                )),
            },
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

impl<'s> TryInto<&'s str> for &'s FormDataItem<'s> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<&'s str, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name } => match value {
                Some(v) => Ok(v),
                None => Err(HttpFailResult::required_parameter_is_missing(
                    name,
                    SRC_FORM_DATA,
                )),
            },
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

impl<'s> TryInto<DateTimeAsMicroseconds> for &'s FormDataItem<'s> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<DateTimeAsMicroseconds, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name } => match value {
                Some(value) => crate::convert_from_str::to_date_time(name, value, SRC_FORM_DATA),
                None => Err(HttpFailResult::required_parameter_is_missing(
                    name,
                    SRC_FORM_DATA,
                )),
            },

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

impl<'s, TValue> TryInto<HashMap<String, TValue>> for &'s FormDataItem<'s>
where
    TValue: DeserializeOwned,
{
    type Error = HttpFailResult;

    fn try_into(self) -> Result<HashMap<String, TValue>, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name } => {
                if value.is_none() {
                    return Ok(HashMap::new());
                }

                return Ok(crate::convert_from_str::to_json_from_str(
                    name,
                    value,
                    SRC_FORM_DATA,
                )?);
            }
            FormDataItem::File {
                name,
                file_name: _,
                content_type: _,
                content,
            } => Ok(crate::convert_from_str::to_json(
                name,
                &Some(content),
                SRC_FORM_DATA,
            )?),
        }
    }
}

impl<'s, TValue> TryInto<Vec<TValue>> for &'s FormDataItem<'s>
where
    TValue: DeserializeOwned,
{
    type Error = HttpFailResult;

    fn try_into(self) -> Result<Vec<TValue>, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name } => {
                if value.is_none() {
                    return Ok(vec![]);
                }
                crate::convert_from_str::to_json_from_str(name, value, SRC_FORM_DATA)
            }
            FormDataItem::File {
                name,
                file_name: _,
                content_type: _,
                content,
            } => crate::convert_from_str::to_json(name, &Some(*content), SRC_FORM_DATA),
        }
    }
}

impl<'s, T: DeserializeOwned> TryInto<RawDataTyped<T>> for FormDataItem<'s> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawDataTyped<T>, Self::Error> {
        match self {
            FormDataItem::ValueAsString { value, name } => match value {
                Some(value) => Ok(RawDataTyped::new(value.as_bytes().to_vec(), SRC_FORM_DATA)),
                None => Err(HttpFailResult::required_parameter_is_missing(
                    name,
                    SRC_FORM_DATA,
                )),
            },

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
            FormDataItem::ValueAsString { value, name } => match value {
                Some(value) => Ok(RawData::new(value.as_bytes().to_vec())),
                None => Err(HttpFailResult::required_parameter_is_missing(
                    name,
                    SRC_FORM_DATA,
                )),
            },
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
