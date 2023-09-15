use my_http_server_core::HttpFailResult;

use super::documentation::out_results::HttpResult;

pub trait AuthErrorFactory {
    fn get_not_authenticated(&self) -> HttpFailResult;
    fn get_not_authorized(&self, claim_name: String) -> HttpFailResult;
    fn get_global_http_fail_result_types(&self) -> Option<Vec<HttpResult>>;
}
