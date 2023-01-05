use std::collections::HashMap;

use serde::de::DeserializeOwned;

use crate::{HttpFailResult, InputParamValue};

impl<TValue> TryInto<HashMap<String, TValue>> for InputParamValue<'_>
where
    TValue: DeserializeOwned,
{
    type Error = HttpFailResult;

    fn try_into(self) -> Result<HashMap<String, TValue>, Self::Error> {
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
                crate::input_param_value::parse_json_value(src.as_bytes())
            }
            InputParamValue::Raw { src, .. } => Err(HttpFailResult::as_not_supported_content_type(
                format!("reading file, but request contains a raw value in {}", src),
            )),
            InputParamValue::File { file, src: _ } => match serde_json::from_slice(&file.content) {
                Ok(result) => Ok(result),
                Err(_) => {
                    let slice = if file.content.len() > 512 {
                        &file.content[0..512]
                    } else {
                        &file.content
                    };

                    let to_show = match std::str::from_utf8(slice) {
                        Ok(result) => result.to_string(),
                        Err(_) => format!("{:?}", slice),
                    };

                    Err(HttpFailResult::invalid_value_to_parse(format!(
                        "Can not parse [{}] as json",
                        to_show
                    )))
                }
            },
        }
    }
}
