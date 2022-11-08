use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct RequestClaim {
    pub id: String,
    pub expires: DateTimeAsMicroseconds,
    pub allowed_ips: Option<Vec<String>>,
}

impl RequestClaim {
    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        if let Some(allowed_ips) = &self.allowed_ips {
            for allowed_ip in allowed_ips {
                if allowed_ip == ip {
                    return true;
                }
            }

            return false;
        }

        true
    }
}

pub trait RequestCredentials {
    fn get_id(&self) -> &str;
    fn get_claims(&self) -> Option<&[RequestClaim]>;
}
