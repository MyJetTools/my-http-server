#[derive(Debug, Clone)]
pub enum HttpParameterInputSource {
    Path,
    Query,
    Header,
    BodyModel,
    BodyRaw,
    FormData,
}

impl HttpParameterInputSource {
    pub fn is_query(&self) -> bool {
        match self {
            HttpParameterInputSource::Query => true,
            _ => false,
        }
    }

    pub fn is_body_raw(&self) -> bool {
        match self {
            HttpParameterInputSource::BodyModel => true,
            _ => false,
        }
    }

    pub fn is_body(&self) -> bool {
        match self {
            HttpParameterInputSource::BodyModel => true,
            HttpParameterInputSource::BodyRaw => true,
            _ => false,
        }
    }

    pub fn is_form_data(&self) -> bool {
        match self {
            HttpParameterInputSource::FormData => true,
            _ => false,
        }
    }

    pub fn is_header(&self) -> bool {
        match self {
            HttpParameterInputSource::Header => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            HttpParameterInputSource::Path => "path",
            HttpParameterInputSource::Query => "query",
            HttpParameterInputSource::Header => "header",
            HttpParameterInputSource::BodyModel => "body",
            HttpParameterInputSource::BodyRaw => "body",
            HttpParameterInputSource::FormData => "form_data",
        }
    }
}
