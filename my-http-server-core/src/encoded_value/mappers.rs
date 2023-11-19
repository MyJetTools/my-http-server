use std::collections::HashMap;

use crate::{
    types::{RawData, RawDataTyped},
    EncodedParamValue, HttpFailResult,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::de::DeserializeOwned;

impl TryInto<DateTimeAsMicroseconds> for EncodedParamValue<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<DateTimeAsMicroseconds, Self::Error> {
        match self {
            Self::UrlEncodedValue { value, src } => {
                let result = value.as_str_or_string();
                let value = crate::url_encoded_data::convert_error(value.get_name(), result, src)?;

                crate::convert_from_str::to_date_time(value.as_str(), value.as_str(), src)
            }
            Self::JsonEncodedData { value, .. } => {
                return value.as_date_time();
            }
        }
    }
}

impl TryInto<bool> for EncodedParamValue<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<bool, Self::Error> {
        return crate::convert_from_str::to_bool(
            self.get_name(),
            self.get_raw_str()?,
            self.get_src(),
        );
    }
}

impl TryInto<u8> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u8, Self::Error> {
        self.from_str()
    }
}

impl TryInto<i8> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i8, Self::Error> {
        self.from_str()
    }
}

impl TryInto<u16> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u16, Self::Error> {
        self.from_str()
    }
}

impl TryInto<i16> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i16, Self::Error> {
        self.from_str()
    }
}

impl TryInto<u32> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u32, Self::Error> {
        self.from_str()
    }
}

impl TryInto<i32> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i32, Self::Error> {
        self.from_str()
    }
}

impl TryInto<u64> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u64, Self::Error> {
        self.from_str()
    }
}

impl TryInto<i64> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i64, Self::Error> {
        self.from_str()
    }
}

impl TryInto<usize> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<usize, Self::Error> {
        self.from_str()
    }
}

impl TryInto<isize> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<isize, Self::Error> {
        self.from_str()
    }
}

impl TryInto<f64> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f64, Self::Error> {
        self.from_str()
    }
}

impl TryInto<f32> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f32, Self::Error> {
        self.from_str()
    }
}

impl TryInto<String> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<String, Self::Error> {
        self.as_string()
    }
}

impl<T: DeserializeOwned> TryInto<Vec<T>> for EncodedParamValue<'_> {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        self.from_json()
    }
}

impl<TValue> TryInto<HashMap<String, TValue>> for EncodedParamValue<'_>
where
    TValue: DeserializeOwned,
{
    type Error = HttpFailResult;

    fn try_into(self) -> Result<HashMap<String, TValue>, Self::Error> {
        match self {
            Self::UrlEncodedValue { src, .. } => {
                Err(HttpFailResult::as_not_supported_content_type(format!(
                    "reading file, but request contains a raw value in {}",
                    src
                )))
            }
            Self::JsonEncodedData { name, value, src } => {
                crate::convert_from_str::to_json(name, value.as_bytes()?, src)
            }
        }
    }
}

impl<'s, T: DeserializeOwned> TryInto<RawDataTyped<T>> for EncodedParamValue<'s> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<RawDataTyped<T>, Self::Error> {
        match self {
            Self::UrlEncodedValue { value, src } => {
                Ok(RawDataTyped::from_slice(value.value.as_bytes(), src))
            }
            Self::JsonEncodedData {
                name: _,
                value,
                src,
            } => Ok(RawDataTyped::from_slice(value.as_bytes()?, src)),
        }
    }
}

impl TryInto<RawData> for EncodedParamValue<'_> {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<RawData, Self::Error> {
        match self {
            Self::UrlEncodedValue { value, .. } => Ok(RawData::from_slice(value.value.as_bytes())),
            Self::JsonEncodedData { value, .. } => Ok(RawData::from_slice(value.as_bytes()?)),
        }
    }
}
