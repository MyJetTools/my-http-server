use super::{HttpParameterType, SwaggerParameterInputSource};

pub struct SwaggerInputParameter {
    pub name: String,
    pub param_type: HttpParameterType,
    pub description: String,
    pub source: SwaggerParameterInputSource,
    pub required: bool,
}
