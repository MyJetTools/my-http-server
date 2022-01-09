use super::{SwaggerParameterInputSource, SwaggerParameterType};

pub struct SwaggerInputParameter {
    pub name: String,
    pub param_type: SwaggerParameterType,
    pub description: String,
    pub source: SwaggerParameterInputSource,
    pub required: bool,
}
