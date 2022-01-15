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
