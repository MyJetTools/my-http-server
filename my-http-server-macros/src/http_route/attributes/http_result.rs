use super::HttpResultModel;

pub struct HttpResult {
    pub status_code: u16,
    pub description: String,
    pub result_type: Option<HttpResultModel>,
}
