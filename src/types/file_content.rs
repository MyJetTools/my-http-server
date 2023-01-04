use crate::{HttpFailResult, InputParamValue};

pub struct FileContent {
    pub content_type: String,
    pub file_name: String,
    pub content: Vec<u8>,
}

impl TryInto<FileContent> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<FileContent, Self::Error> {
        match self {
            InputParamValue::UrlEncodedValueAsStringRef { src, .. } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "reading file, but request contains a raw value in {}",
                    src
                )))
            }
            InputParamValue::UrlEncodedValueAsString { src, .. } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "reading file, but request contains a raw value in {}",
                    src
                )))
            }
            InputParamValue::JsonEncodedData { src, .. } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "reading file, but request contains a raw value in {}",
                    src
                )))
            }
            InputParamValue::Raw { src, .. } => Err(HttpFailResult::as_not_supported_content_type(
                format!("reading file, but request contains a raw value in {}", src),
            )),
            InputParamValue::File { file, src: _ } => Ok(file),
        }
    }
}
