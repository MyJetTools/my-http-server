use super::{types::HttpParameterType, HttpParameterInputSource};

pub struct HttpInputParameter {
    pub name: String,
    pub param_type: HttpParameterType,
    pub description: String,
    pub source: HttpParameterInputSource,
    pub required: bool,
}
