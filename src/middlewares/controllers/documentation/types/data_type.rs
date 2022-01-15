use super::{ArrayElement, HttpObjectDescription, HttpSimpleType};

pub enum HttpDataType {
    SimpleType(HttpSimpleType),
    Object(HttpObjectDescription),
    ArrayOf(ArrayElement),
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
        Self::SimpleType(HttpSimpleType::Boolean)
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

    pub fn as_password() -> Self {
        Self::SimpleType(HttpSimpleType::Password)
    }

    pub fn as_object(struct_id: &str) -> HttpObjectDescription {
        HttpObjectDescription::new(struct_id)
    }
}
