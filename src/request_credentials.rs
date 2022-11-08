use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct RequestClaim {
    pub id: String,
    pub expires: DateTimeAsMicroseconds,
    pub allowed_ips: Option<Vec<String>>,
}

pub trait RequestCredentials {
    fn get_id(&self) -> &str;
    fn get_claims(&self) -> Option<&[RequestClaim]>;
}
