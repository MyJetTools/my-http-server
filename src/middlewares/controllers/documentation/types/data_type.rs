use super::{HttpObjectDescription, HttpParameterType};

pub enum HttpDataType {
    SimpleType(HttpParameterType),
    Object(HttpObjectDescription),
    None,
}

impl HttpDataType {
    pub fn as_string() -> Self {
        Self::SimpleType(HttpParameterType::String)
    }

    pub fn as_integer() -> Self {
        Self::SimpleType(HttpParameterType::Integer)
    }

    pub fn as_long() -> Self {
        Self::SimpleType(HttpParameterType::Boolean)
    }

    pub fn as_float() -> Self {
        Self::SimpleType(HttpParameterType::Float)
    }

    pub fn as_double() -> Self {
        Self::SimpleType(HttpParameterType::Double)
    }

    pub fn as_binary() -> Self {
        Self::SimpleType(HttpParameterType::Binary)
    }

    pub fn as_date() -> Self {
        Self::SimpleType(HttpParameterType::Date)
    }

    pub fn as_date_time() -> Self {
        Self::SimpleType(HttpParameterType::DateTime)
    }

    pub fn as_password() -> Self {
        Self::SimpleType(HttpParameterType::Password)
    }

    pub fn as_object(struct_id: &str) -> Self {
        Self::Object(HttpObjectDescription::new(struct_id))
    }
}
