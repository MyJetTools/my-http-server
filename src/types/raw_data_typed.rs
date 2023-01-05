use std::marker::PhantomData;

use serde::de::DeserializeOwned;

use crate::{HttpFailResult, HttpRequestBody, InputParamValue};

pub struct RawData<T: DeserializeOwned> {
    data: Vec<u8>,
    ty: PhantomData<T>,
}

impl<T: DeserializeOwned> RawData<T> {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            ty: PhantomData,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn deserialize_json(&self) -> Result<T, HttpFailResult> {
        crate::input_param_value::parse_json_value(&self.data)
    }
}

impl<T: DeserializeOwned> AsRef<[u8]> for RawData<T> {
    fn as_ref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<T: DeserializeOwned> TryInto<RawData<T>> for InputParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<RawData<T>, Self::Error> {
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

impl<T: DeserializeOwned> Into<RawData<T>> for HttpRequestBody {
    fn into(self) -> RawData<T> {
        RawData::new(self.get_body())
    }
}
