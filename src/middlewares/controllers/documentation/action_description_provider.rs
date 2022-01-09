use crate::WebContentType;

use super::HttpInputParameter;

pub struct HttpActionDescription<'s> {
    pub name: &'s str,
    pub description: &'s str,
    pub out_content_type: WebContentType,
    pub input_params: Option<Vec<HttpInputParameter>>,
}

pub trait HttpActionDescriptionProvider {
    fn get_controller_description(&self) -> Option<HttpActionDescription>;
}
