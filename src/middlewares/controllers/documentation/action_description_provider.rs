use super::{in_parameters::HttpInputParameter, out_results::HttpResult};

pub struct HttpActionDescription<'s> {
    pub name: &'s str,
    pub description: &'s str,
    pub input_params: Option<Vec<HttpInputParameter>>,
    pub results: Vec<HttpResult>,
}

pub trait HttpActionDescriptionProvider {
    fn get_description(&self) -> Option<HttpActionDescription>;
}
