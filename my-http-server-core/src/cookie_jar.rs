use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};

pub struct CookieJarCooke {
    pub name: String,
    pub value: String,
    pub expires_at: DateTimeAsMicroseconds,
}

impl CookieJarCooke {
    pub fn to_string(&self) -> String {
        format!(
            "{}={}; Expires={}",
            self.name,
            self.value,
            self.expires_at.to_rfc7231()
        )
    }
}

pub struct CookieJar {
    cookies: Vec<CookieJarCooke>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: Vec::new(),
        }
    }

    pub fn set_cookie<'s>(
        &mut self,
        name: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'s>>,
        expires_at: DateTimeAsMicroseconds,
    ) {
        let name = name.into();
        let value = value.into();
        self.cookies.push(CookieJarCooke {
            name: name.to_string(),
            value: value.to_string(),
            expires_at,
        });
    }

    pub fn get_cookies(&self) -> impl Iterator<Item = &CookieJarCooke> {
        self.cookies.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}
