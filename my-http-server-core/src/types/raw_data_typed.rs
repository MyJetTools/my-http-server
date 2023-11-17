use std::marker::PhantomData;

use rust_extensions::StrOrString;
use serde::de::DeserializeOwned;

use crate::{HttpFailResult, HttpRequestBody};

pub struct RawDataTyped<'s, T: DeserializeOwned> {
    name: StrOrString<'s>,
    data: Vec<u8>,
    ty: PhantomData<T>,
    src: &'static str,
}

impl<'s, T: DeserializeOwned> RawDataTyped<'s, T> {
    pub fn new(name: StrOrString<'s>, data: Vec<u8>, src: &'static str) -> Self {
        Self {
            name,
            data,
            ty: PhantomData,
            src,
        }
    }

    pub fn from_slice(name: StrOrString<'s>, data: &[u8], src: &'static str) -> Self {
        Self {
            name,
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

impl<'s, T: DeserializeOwned> AsRef<[u8]> for RawDataTyped<'s, T> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<'s, T: DeserializeOwned> TryInto<RawDataTyped<'s, T>> for HttpRequestBody {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawDataTyped<'s, T>, Self::Error> {
        Ok(RawDataTyped::new("RawBody".into(), self.get_body(), "Body"))
    }
}
