use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::de::DeserializeOwned;

use crate::{
    data_src::SRC_HEADER,
    types::{RawData, RawDataTyped},
    HttpFailResult,
};

pub struct HeaderValue<'s> {
    pub name: &'static str,
    pub value: &'s [u8],
}

impl<'s> HeaderValue<'s> {
    pub fn new(name: &'static str, value: &'s [u8]) -> Self {
        Self { name, value }
    }

    pub fn from_header_value(name: &'static str, value: &'s hyper::header::HeaderValue) -> Self {
        Self {
            name,
            value: value.as_bytes(),
        }
    }

    pub fn as_str(&self) -> Result<&'s str, HttpFailResult> {
        let result = std::str::from_utf8(self.value);
        match result {
            Ok(result) => Ok(result),
            Err(_) => Err(HttpFailResult::invalid_value_to_parse(format!(
                "Can not parse header value in {}",
                SRC_HEADER
            ))),
        }
    }
}

impl<TValue> TryInto<HashMap<String, TValue>> for HeaderValue<'_>
where
    TValue: DeserializeOwned,
{
    type Error = HttpFailResult;

    fn try_into(self) -> Result<HashMap<String, TValue>, Self::Error> {
        crate::convert_from_str::to_json(self.name, &Some(self.value), SRC_HEADER)
    }
}

impl<'s, T: DeserializeOwned> TryInto<RawDataTyped<T>> for HeaderValue<'s> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawDataTyped<T>, Self::Error> {
        Ok(RawDataTyped::new(self.value.to_vec(), SRC_HEADER))
    }
}

impl TryInto<RawData> for HeaderValue<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<RawData, Self::Error> {
        Ok(RawData::new(self.value.to_vec()))
    }
}

impl TryInto<String> for HeaderValue<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<String, Self::Error> {
        Ok(self.as_str()?.to_string())
    }
}

impl<'s> TryInto<&'s str> for HeaderValue<'s> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<&'s str, Self::Error> {
        Ok(self.as_str()?)
    }
}

impl TryInto<DateTimeAsMicroseconds> for HeaderValue<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<DateTimeAsMicroseconds, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_date_time(self.name, value, SRC_HEADER)
    }
}

impl TryInto<u8> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u8, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<bool> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<bool, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_bool(self.name, value, SRC_HEADER)
    }
}

impl TryInto<i8> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i8, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<u16> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u16, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<i16> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i16, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<u32> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u32, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<i32> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i32, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<u64> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u64, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<i64> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i64, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<usize> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<usize, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<isize> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<isize, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<f32> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f32, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}

impl TryInto<f64> for HeaderValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f64, Self::Error> {
        let value = self.as_str()?;
        crate::convert_from_str::to_simple_value(self.name, value, SRC_HEADER)
    }
}
