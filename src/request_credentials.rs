use crate::HttpRequest;

pub trait RequestCredentials {
    fn get_id(&self) -> &str;
    fn get_claim(&self, request: &HttpRequest, claim_id: &str) -> Option<&str>;
}
