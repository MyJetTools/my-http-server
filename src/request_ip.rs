use rust_extensions::StrOrString;

pub enum RequestIp<'s> {
    SingleIp(StrOrString<'s>),
    Forwarded(Vec<&'s str>),
}

impl<'s> RequestIp<'s> {
    pub fn create_as_single_ip(ip: String) -> Self {
        let mut value = StrOrString::create_as_string(ip);

        extract_ip(&mut value);
        Self::SingleIp(value)
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

fn extract_ip(src: &mut StrOrString) {
    if let Some(pos) = src.as_str().find(|itm| itm == ':') {
        src.slice_it(None, pos.into())
    }
}
