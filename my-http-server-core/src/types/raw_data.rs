use crate::{HttpFailResult, HttpRequestBodyContent};

pub struct RawData(Vec<u8>);

impl RawData {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Self(data.to_vec())
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl Into<Vec<u8>> for RawData {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for RawData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryInto<RawData> for HttpRequestBodyContent {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawData, Self::Error> {
        Ok(RawData::new(self.get_body()))
    }
}
