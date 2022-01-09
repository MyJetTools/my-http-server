pub enum HttpParameterInputSource {
    Path,
    Query,
    Header,
    FormData,
}

impl HttpParameterInputSource {
    pub fn to_str(&self) -> &str {
        match self {
            HttpParameterInputSource::Path => "path",
            HttpParameterInputSource::Query => "query",
            HttpParameterInputSource::Header => "header",
            HttpParameterInputSource::FormData => "formData",
        }
    }
}
