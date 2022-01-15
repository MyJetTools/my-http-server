use super::types::HttpDataType;

pub struct HttpInputParameterData {
    pub name: String,
    pub data_type: HttpDataType,
    pub description: String,
    pub required: bool,
}

pub enum HttpInputParameter {
    Path(HttpInputParameterData),
    Query(HttpInputParameterData),
    Header(HttpInputParameterData),
    FormData(HttpInputParameterData),
    Body(HttpInputParameterData),
}

/*
impl HttpInputParameter {
    pub fn to_str(&self) -> &str {
        match self {
            HttpInputParameter::Path(_) => "path",
            HttpInputParameter::Query(_) => "query",
            HttpInputParameter::Header(_) => "header",
            HttpInputParameter::FormData(_) => "formData",
            HttpInputParameter::Body(_) => "body",
        }
    }
}
 */
