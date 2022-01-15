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
    pub fn as_string(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::String,
        }
    }

    pub fn as_integer(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::Integer,
        }
    }

    pub fn as_long(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::Long,
        }
    }

    pub fn as_float(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::Float,
        }
    }

    pub fn as_double(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::Double,
        }
    }

    pub fn as_binary(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::Binary,
        }
    }

    pub fn as_date(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::Date,
        }
    }

    pub fn as_date_time(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::DateTime,
        }
    }

    pub fn as_password(required: bool) -> Self {
        Self::SimpleType {
            required,
            param_type: HttpParameterType::Password,
        }
    }

    pub fn as_object(struct_id: &str, required: bool) -> Self {
        Self::Object {
            required,
            object_description: HttpObjectDescription::new(struct_id),
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
