pub trait RequestCredentials {
    fn get_id(&self) -> &str;
    fn get_claim(&self, claim_id: &str) -> Option<&str>;
}
