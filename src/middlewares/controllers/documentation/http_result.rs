use super::types::HttpDataType;

pub struct HttpResult {
    pub http_code: u16,
    pub nullable: bool,
    pub description: String,
    pub data_type: HttpDataType,
}
