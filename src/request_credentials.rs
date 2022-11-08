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

impl<'s> RequestClaim<'s> {
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
