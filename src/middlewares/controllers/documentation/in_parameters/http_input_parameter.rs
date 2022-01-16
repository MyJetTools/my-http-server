use crate::middlewares::controllers::documentation::data_types::HttpField;

pub struct HttpInputParameter {
    pub field: HttpField,
    pub description: String,
    pub required: bool,
    pub source: HttpParameterInputSource,
}

pub enum HttpParameterInputSource {
    Path,
    Query,
    Header,
    FormData,
    Body,
}

impl HttpParameterInputSource {
    pub fn as_str(&self) -> &str {
        match self {
            HttpParameterInputSource::Path => "path",
            HttpParameterInputSource::Query => "query",
            HttpParameterInputSource::Header => "header",
            HttpParameterInputSource::FormData => "formData",
            HttpParameterInputSource::Body => "body",
        }
    }
}
