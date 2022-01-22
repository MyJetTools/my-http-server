use super::{ArrayElement, HttpEnumStructure, HttpObjectStructure, HttpSimpleType};

pub enum HttpDataType {
    SimpleType(HttpSimpleType),
    Object(HttpObjectStructure),
    ObjectId { struct_id: String },
    ArrayOf(ArrayElement),
    Enum(HttpEnumStructure),
    None,
}

impl HttpDataType {
    pub fn as_string() -> Self {
        Self::SimpleType(HttpSimpleType::String)
    }

    pub fn as_integer() -> Self {
        Self::SimpleType(HttpSimpleType::Integer)
    }

    pub fn as_long() -> Self {
        Self::SimpleType(HttpSimpleType::Long)
    }

    pub fn as_float() -> Self {
        Self::SimpleType(HttpSimpleType::Float)
    }

    pub fn as_double() -> Self {
        Self::SimpleType(HttpSimpleType::Double)
    }

    pub fn as_binary() -> Self {
        Self::SimpleType(HttpSimpleType::Binary)
    }

    pub fn as_date() -> Self {
        Self::SimpleType(HttpSimpleType::Date)
    }

    pub fn as_date_time() -> Self {
        Self::SimpleType(HttpSimpleType::DateTime)
    }

    pub fn as_bool() -> Self {
        Self::SimpleType(HttpSimpleType::Boolean)
    }

    pub fn as_boolean() -> Self {
        Self::SimpleType(HttpSimpleType::Boolean)
    }

    pub fn as_password() -> Self {
        Self::SimpleType(HttpSimpleType::Password)
    }

    pub fn as_object(struct_id: &str) -> Self {
        Self::ObjectId {
            struct_id: struct_id.to_string(),
        }
    }

    pub fn as_array_of_object(struct_id: &str) -> Self {
        Self::ArrayOf(ArrayElement::ObjectId {
            struct_id: struct_id.to_string(),
        })
    }
}
