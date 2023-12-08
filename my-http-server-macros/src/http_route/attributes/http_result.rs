use types_reader::TokensObject;

use super::HttpResultModel;

pub struct HttpResult {
    pub status_code: u16,
    pub description: String,
    pub result_type: Option<HttpResultModel>,
}

impl HttpResult {
    pub fn new(param_list: &TokensObject) -> Result<HttpResult, syn::Error> {
        let result = HttpResult {
            status_code: param_list.get_named_param("status_code")?.try_into()?,
            description: param_list.get_named_param("description")?.try_into()?,
            result_type: HttpResultModel::new(param_list)?,
        };

        Ok(result)
    }
}
