use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub expires_at: Option<DateTimeAsMicroseconds>,
    pub max_age: Option<u64>,
    pub domain: Option<StrOrString<'static>>,
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            expires_at: None,
            max_age: None,
            domain: None,
        }
    }
    pub fn set_expires_at(mut self, expires_at: DateTimeAsMicroseconds) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn set_max_age(mut self, max_age: u64) -> Self {
        self.max_age = Some(max_age);
        self
    }

    pub fn set_domain(mut self, domain: impl Into<StrOrString<'static>>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("{}={}", self.name, self.value);

        if let Some(expires_at) = self.expires_at {
            add_element(&mut result, "Expires", expires_at.to_rfc7231().as_str());
        }

        if let Some(max_age) = self.max_age {
            add_element(&mut result, "Max-Age", max_age.to_string().as_str());
        }

        if let Some(domain) = &self.domain {
            add_element(&mut result, "Domain", domain.as_str());
        }

        result
    }
}

impl Into<Cookie> for (String, String) {
    fn into(self) -> Cookie {
        Cookie {
            name: self.0,
            value: self.1,
            expires_at: None,
            max_age: None,
            domain: None,
        }
    }
}

impl Into<Cookie> for (&'static str, String) {
    fn into(self) -> Cookie {
        Cookie {
            name: self.0.to_string(),
            value: self.1,
            expires_at: None,
            max_age: None,
            domain: None,
        }
    }
}

impl Into<Cookie> for (String, String, DateTimeAsMicroseconds) {
    fn into(self) -> Cookie {
        Cookie {
            name: self.0,
            value: self.1,
            expires_at: Some(self.2),
            max_age: None,
            domain: None,
        }
    }
}

impl Into<Cookie> for (&'static str, String, DateTimeAsMicroseconds) {
    fn into(self) -> Cookie {
        Cookie {
            name: self.0.to_string(),
            value: self.1,
            expires_at: Some(self.2),
            max_age: None,
            domain: None,
        }
    }
}
fn add_element(result: &mut String, name: &str, value: &str) {
    result.push_str("; ");
    result.push_str(name);
    result.push('=');
    result.push_str(value);
}
