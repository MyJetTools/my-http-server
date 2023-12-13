use crate::{EncodedParamValue, FormDataItem, HttpFailResult};

pub struct FileContent {
    pub content_type: String,
    pub file_name: String,
    pub content: Vec<u8>,
}

impl<'s> TryInto<FileContent> for EncodedParamValue<'s> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<FileContent, Self::Error> {
        match self {
            EncodedParamValue::UrlEncodedValue {  src, .. } => {
                return Err(HttpFailResult::as_forbidden(Some(format!(
                    "[{src}] Can not convert Url encoded value into file",
                ))));
            }
            EncodedParamValue::JsonEncodedData {  src,.. } => {
                return Err(HttpFailResult::as_forbidden(Some(format!(
                    "[{src}] Can not convert Json encoded value into file",
                ))));
            }
            EncodedParamValue::FormData { name:_, value } => match value {
                FormDataItem::ValueAsString { value: _, name } => {
                    Err(HttpFailResult::as_not_supported_content_type(format!(
                        "Field {} for FormData contains a value which is not possible to convert to a file",
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
            },
        }
    }
}
