use crate::{HttpFailResult, HttpRequestBody};

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
}

impl AsRef<[u8]> for RawData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryInto<RawData> for HttpRequestBody {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawData, Self::Error> {
        Ok(RawData::new(self.get_body()))
    }
}
