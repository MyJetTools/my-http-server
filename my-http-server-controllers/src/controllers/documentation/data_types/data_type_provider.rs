use std::collections::{BTreeMap, HashMap};

use my_http_server_core::types::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::de::DeserializeOwned;

use super::{ArrayElement, HttpDataType, HttpSimpleType};

pub trait DataTypeProvider {
    fn get_data_type() -> HttpDataType;
    fn get_generic_type() -> Option<&'static str> {
        None
    }
}

impl DataTypeProvider for u8 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Byte)
    }
}

impl DataTypeProvider for i8 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Byte)
    }
}

impl DataTypeProvider for u16 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Integer)
    }
}

impl DataTypeProvider for i16 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Integer)
    }
}

impl DataTypeProvider for u32 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Integer)
    }
}

impl DataTypeProvider for i32 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Integer)
    }
}

impl DataTypeProvider for u64 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Long)
    }
}

impl DataTypeProvider for i64 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Long)
    }
}

impl DataTypeProvider for usize {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Long)
    }
}

impl DataTypeProvider for isize {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Long)
    }
}

impl DataTypeProvider for f32 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Double)
    }
}

impl DataTypeProvider for f64 {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Double)
    }
}

impl DataTypeProvider for bool {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Boolean)
    }
}

impl DataTypeProvider for String {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::String)
    }
}

impl<'s> DataTypeProvider for &'s str {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::String)
    }
}

impl<'s> DataTypeProvider for DateTimeAsMicroseconds {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::DateTime)
    }
}

impl<T: DataTypeProvider> DataTypeProvider for Vec<T> {
    fn get_data_type() -> HttpDataType {
        let data_type = T::get_data_type();
        match data_type {
            HttpDataType::SimpleType(tp) => HttpDataType::ArrayOf(ArrayElement::SimpleType(tp)),
            HttpDataType::Object(obj) => HttpDataType::ArrayOf(ArrayElement::Object(obj)),
            HttpDataType::Enum(item) => HttpDataType::ArrayOf(ArrayElement::Enum(item)),
            _ => panic!("Unsupported data type: {:?}", data_type),
        }
    }
}

impl<TValue: DataTypeProvider> DataTypeProvider for HashMap<String, TValue> {
    fn get_data_type() -> HttpDataType {
        match TValue::get_data_type() {
            HttpDataType::SimpleType(tp) => {
                HttpDataType::DictionaryOf(ArrayElement::SimpleType(tp))
            }
            HttpDataType::Object(obj) => HttpDataType::DictionaryOf(ArrayElement::Object(obj)),
            HttpDataType::ArrayOf(item) => HttpDataType::DictionaryOfArray(item),
            _ => panic!("Unsupported data type {:?}", TValue::get_data_type()),
        }
    }
}

impl<TValue: DataTypeProvider> DataTypeProvider for BTreeMap<String, TValue> {
    fn get_data_type() -> HttpDataType {
        match TValue::get_data_type() {
            HttpDataType::SimpleType(tp) => {
                HttpDataType::DictionaryOf(ArrayElement::SimpleType(tp))
            }
            HttpDataType::Object(obj) => HttpDataType::DictionaryOf(ArrayElement::Object(obj)),
            HttpDataType::ArrayOf(item) => HttpDataType::DictionaryOfArray(item),
            _ => panic!("Unsupported data type {:?}", TValue::get_data_type()),
        }
    }
}

impl DataTypeProvider for FileContent {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Binary)
    }
}

impl DataTypeProvider for RawData {
    fn get_data_type() -> HttpDataType {
        HttpDataType::SimpleType(HttpSimpleType::Binary)
    }
}

impl<T: DeserializeOwned + DataTypeProvider> DataTypeProvider for RawDataTyped<T> {
    fn get_data_type() -> HttpDataType {
        T::get_data_type()
    }
}
