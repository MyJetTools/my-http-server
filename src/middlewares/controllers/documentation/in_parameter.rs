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
