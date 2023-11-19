use std::marker::PhantomData;

use serde::de::DeserializeOwned;

use crate::{HttpFailResult, HttpRequestBody};

pub struct RawDataTyped<T: DeserializeOwned> {
    data: Vec<u8>,
    ty: PhantomData<T>,
    src: &'static str,
}

impl<T: DeserializeOwned> RawDataTyped<T> {
    pub fn new(data: Vec<u8>, src: &'static str) -> Self {
        Self {
            data,
            ty: PhantomData,
            src,
        }
    }

    pub fn from_slice(data: &[u8], src: &'static str) -> Self {
        Self {
            data: data.to_vec(),
            ty: PhantomData,
            src,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn deserialize_json(&self) -> Result<T, HttpFailResult> {
        crate::convert_from_str::to_json("RawDataType", &self.data, self.src)
    }
}

impl<T: DeserializeOwned> AsRef<[u8]> for RawDataTyped<T> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<T: DeserializeOwned> TryInto<RawDataTyped<T>> for HttpRequestBody {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawDataTyped<T>, Self::Error> {
        Ok(RawDataTyped::new(self.get_body(), "Body"))
    }
}
