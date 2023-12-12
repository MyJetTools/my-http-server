use crate::controllers::RequiredClaims;

use super::{in_parameters::HttpParameters, out_results::HttpResult};

#[derive(Debug, Clone)]
pub enum ShouldBeAuthorized {
    Yes,
    YesWithClaims(RequiredClaims),
    No,
    UseGlobal,
}

pub struct HttpActionDescription<'s> {
    pub controller_name: &'s str,
    pub summary: &'s str,
    pub description: &'s str,
    pub input_params: HttpParameters,
    pub results: Vec<HttpResult>,
    pub should_be_authorized: ShouldBeAuthorized,
}

pub trait HttpActionDescriptionProvider {
    fn get_description(&self) -> Option<HttpActionDescription>;
}
