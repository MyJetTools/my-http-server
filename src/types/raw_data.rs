use crate::{HttpFailResult, InputParamValue};

pub struct RawData(Vec<u8>);

impl RawData {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl TryInto<RawData> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<RawData, Self::Error> {
        match self {
            InputParamValue::UrlEncodedValueAsStringRef { src, .. } => {
                Ok(RawData(src.as_bytes().to_vec()))
            }
            InputParamValue::UrlEncodedValueAsString { src, .. } => {
                Ok(RawData(src.as_bytes().to_vec()))
            }
            InputParamValue::JsonEncodedData { src, .. } => Ok(RawData(src.as_bytes().to_vec())),
            InputParamValue::Raw { src, .. } => Ok(RawData(src.as_bytes().to_vec())),
            InputParamValue::File { file, src: _ } => Ok(RawData(file.content)),
        }
    }
}
