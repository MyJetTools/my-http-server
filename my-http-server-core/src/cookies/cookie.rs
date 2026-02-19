use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

#[derive(Debug)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub expires_at: Option<DateTimeAsMicroseconds>,
    pub max_age: Option<u64>,
    pub domain: Option<StrOrString<'static>>,
    pub path: Option<StrOrString<'static>>,
    pub http_only: bool,
    pub partitioned: bool,
    pub secure: bool,
    pub same_site: bool,
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            expires_at: None,
            max_age: None,
            domain: None,
            http_only: false,
            partitioned: false,
            path: None,
            secure: false,
            same_site: false,
        }
    }

    pub fn new_common_case(
        name: impl Into<String>,
        value: impl Into<String>,
        domain: impl Into<StrOrString<'static>>,
        expiration: DateTimeAsMicroseconds,
    ) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            expires_at: Some(expiration),
            max_age: None,
            domain: Some(domain.into()),
            http_only: true,
            partitioned: false,
            path: Some("/".into()),
            secure: true,
            same_site: true,
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

    pub fn set_secure(mut self) -> Self {
        self.secure = true;
        self
    }

    pub fn set_same_site(mut self) -> Self {
        self.same_site = true;
        self
    }

    pub fn set_path(mut self, path: impl Into<StrOrString<'static>>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn set_http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    pub fn set_partitioned(mut self) -> Self {
        self.partitioned = true;
        self
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("{}={}", self.name, self.value);

        if let Some(path) = &self.path {
            add_element(&mut result, "Path", path.as_str());
        }

        if let Some(expires_at) = self.expires_at {
            add_element(&mut result, "Expires", expires_at.to_rfc7231().as_str());
        }

        if let Some(max_age) = self.max_age {
            add_element(&mut result, "Max-Age", max_age.to_string().as_str());
        }

        if let Some(domain) = &self.domain {
            add_element(&mut result, "Domain", domain.as_str());
        }

        if self.http_only {
            add_empty_element(&mut result, "HttpOnly");
        }

        if self.partitioned {
            add_empty_element(&mut result, "Partitioned");
        }

        if self.secure {
            add_empty_element(&mut result, "Secure");
        }

        if self.same_site {
            add_element(&mut result, "Samesite", "None");
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
            http_only: false,
            partitioned: false,
            path: None,
            same_site: false,
            secure: false,
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
            http_only: false,
            partitioned: false,
            path: None,
            same_site: false,
            secure: false,
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
            http_only: false,
            partitioned: false,
            path: None,
            same_site: false,
            secure: false,
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
            http_only: false,
            partitioned: false,
            path: None,
            same_site: false,
            secure: false,
        }
    }
}
fn add_element(result: &mut String, name: &str, value: &str) {
    result.push_str("; ");
    result.push_str(name);
    result.push('=');
    result.push_str(value);
}

fn add_empty_element(result: &mut String, name: &str) {
    result.push_str("; ");
    result.push_str(name);
}
