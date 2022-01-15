use super::types::HttpDataType;

pub struct HttpInputParameter {
    pub name: String,
    pub data_type: HttpDataType,
    pub description: String,
    pub required: bool,
    pub src: HttpParameterInputSource,
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
