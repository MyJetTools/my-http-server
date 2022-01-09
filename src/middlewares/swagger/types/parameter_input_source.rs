pub enum SwaggerParameterInputSource {
    Path,
    Query,
    Headers,
    FormData,
}

impl SwaggerParameterInputSource {
    pub fn to_str(&self) -> &str {
        match self {
            SwaggerParameterInputSource::Path => "path",
            SwaggerParameterInputSource::Query => "query",
            SwaggerParameterInputSource::Headers => "headers",
            SwaggerParameterInputSource::FormData => "formData",
        }
    }
}
