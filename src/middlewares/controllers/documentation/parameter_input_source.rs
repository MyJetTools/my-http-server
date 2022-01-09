pub enum HttpParameterInputSource {
    Path,
    Query,
    Headers,
    FormData,
}

impl HttpParameterInputSource {
    pub fn to_str(&self) -> &str {
        match self {
            HttpParameterInputSource::Path => "path",
            HttpParameterInputSource::Query => "query",
            HttpParameterInputSource::Headers => "headers",
            HttpParameterInputSource::FormData => "formData",
        }
    }
}
