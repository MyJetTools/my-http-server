pub enum RequestIp<'s> {
    Result(String),
    Forwarded(Vec<&'s str>),
}

impl<'s> RequestIp<'s> {
    pub fn get_real_ip(&'s self) -> &'s str {
        match self {
            RequestIp::Result(ip) => ip.as_str(),
            RequestIp::Forwarded(forwarded_ips) => forwarded_ips[0],
        }
    }

    pub fn get_real_ip_as_string(self) -> String {
        match self {
            RequestIp::Result(ip) => ip,
            RequestIp::Forwarded(forwarded_ips) => forwarded_ips[0].to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RequestIp::Result(ip) => ip.to_string(),
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
