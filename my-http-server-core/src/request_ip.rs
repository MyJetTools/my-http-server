use crate::{HttpRequestHeaders, SocketAddress};

pub enum RequestIp<'s> {
    SingleIp(String),
    Forwarded(Vec<&'s str>),
}

impl<'s> RequestIp<'s> {
    pub fn new(addr: &SocketAddress, headers: &'s impl HttpRequestHeaders) -> Self {
        let x_forwarded_for = headers.try_get_case_sensitive_as_str(crate::X_FORWARDED_FOR_HEADER);

        if let Ok(x_forwarded_for) = x_forwarded_for {
            if let Some(x_forwarded_for) = x_forwarded_for {
                let result: Vec<&str> = x_forwarded_for.split(",").map(|itm| itm.trim()).collect();
                return RequestIp::Forwarded(result);
            }
        }

        return RequestIp::SingleIp(addr.ip_as_string());
    }

    pub fn create_as_single_ip(addr: SocketAddress) -> Self {
        Self::SingleIp(addr.ip_as_string())
    }
    pub fn get_real_ip(&'s self) -> &'s str {
        match self {
            RequestIp::SingleIp(ip) => ip.as_str(),
            RequestIp::Forwarded(forwarded_ips) => forwarded_ips[0],
        }
    }

    pub fn get_real_ip_as_string(self) -> String {
        match self {
            RequestIp::SingleIp(ip) => ip.to_string(),
            RequestIp::Forwarded(forwarded_ips) => forwarded_ips[0].to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RequestIp::SingleIp(ip) => ip.to_string(),
            RequestIp::Forwarded(forwarded_ips) => {
                let mut result = String::new();
                for ip in forwarded_ips {
                    result.push_str(ip);
                    result.push_str(", ");
                }
                result.pop();
                result.pop();
                result
            }
        }
    }
}

/*
fn extract_ip(src: &mut String) {
    if let Some(pos) = src.as_str().find(|itm| itm == ':') {
        for _ in 0..src.len() - pos {
            src.pop();
        }
    }
}
 */
