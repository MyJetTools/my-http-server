use super::{HttpObjectDescription, HttpParameterType};

pub enum HttpDataType {
    SimpleType {
        required: bool,
        param_type: HttpParameterType,
    },
    Object {
        required: bool,
        description: HttpObjectDescription,
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
}
