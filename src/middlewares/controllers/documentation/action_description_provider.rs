use crate::WebContentType;

pub struct HttpActionDescription<'s> {
    pub name: &'s str,
    pub description: &'s str,
    pub out_content_type: WebContentType,
}

pub trait HttpActionDescriptionProvider {
    fn get_controller_description(&self) -> HttpActionDescription;
}
