use super::HttpResult;

pub struct ApiData<'s> {
    pub controller: &'s str,
    pub description: &'s str,
    pub summary: &'s str,
    pub results: Option<Vec<HttpResult>>,
}
