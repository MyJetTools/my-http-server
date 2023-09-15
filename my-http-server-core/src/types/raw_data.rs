use crate::{HttpFailResult, HttpRequestBody, InputParamValue};

pub struct RawData(Vec<u8>);

impl RawData {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl AsRef<[u8]> for RawData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryInto<RawData> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<RawData, Self::Error> {
        match self {
            InputParamValue::UrlEncodedValueAsStringRef { src, .. } => {
                Ok(RawData::new(src.as_bytes().to_vec()))
            }
            InputParamValue::UrlEncodedValueAsString { src, .. } => {
                Ok(RawData::new(src.as_bytes().to_vec()))
            }
            InputParamValue::JsonEncodedData { src, .. } => {
                Ok(RawData::new(src.as_bytes().to_vec()))
            }
            InputParamValue::Raw { src, .. } => Ok(RawData::new(src.as_bytes().to_vec())),
            InputParamValue::File { file, src: _ } => Ok(RawData::new(file.content)),
        }
    }
}

impl TryInto<RawData> for HttpRequestBody {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawData, Self::Error> {
        Ok(RawData::new(self.get_body()))
    }
}
