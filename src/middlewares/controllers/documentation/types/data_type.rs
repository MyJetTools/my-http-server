use super::{HttpObjectDescription, HttpParameterType};

pub enum HttpDataType {
    SimpleType {
        required: bool,
        param_type: HttpParameterType,
    },
    Object {
        required: bool,
        object_description: HttpObjectDescription,
    },
    None,
}

impl HttpDataType {
    pub fn as_string() -> Self {
        Self::SimpleType {
            required: false,
            param_type: HttpParameterType::String,
        }
    }

    pub fn is_required(&self) -> bool {
        match self {
            HttpDataType::SimpleType {
                required,
                param_type: _,
            } => *required,
            HttpDataType::Object {
                required,
                object_description: _,
            } => *required,
            HttpDataType::None => false,
        }
    }
}
