use crate::controllers::RequiredClaims;

use super::{in_parameters::HttpParameters, out_results::HttpResult};

#[derive(Debug, Clone)]
pub enum ShouldBeAuthorized {
    Yes,
    YesWithClaims(RequiredClaims),
    No,
    UseGlobal,
}

pub struct HttpActionDescription {
    pub controller_name: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
    pub input_params: HttpParameters,
    pub results: Vec<HttpResult>,
    pub should_be_authorized: ShouldBeAuthorized,
}

pub trait HttpActionDescriptionProvider {
    fn get_description(&self) -> Option<HttpActionDescription>;
}
