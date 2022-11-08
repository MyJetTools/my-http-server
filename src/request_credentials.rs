use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct RequestClaim<'s> {
    pub id: &'s str,
    pub expires: DateTimeAsMicroseconds,
    pub allowed_ips: &'s Option<Vec<String>>,
}

pub trait RequestCredentials {
    fn get_id(&self) -> &str;
    fn get_claims<'s>(&'s self) -> Option<RequestClaim<'s>>;
}
