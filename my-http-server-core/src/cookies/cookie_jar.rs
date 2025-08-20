use super::*;

#[derive(Default, Debug)]
pub struct CookieJar {
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: Vec::new(),
        }
    }

    pub fn set_cookie<'s>(mut self, cookie: impl Into<Cookie>) -> Self {
        self.cookies.push(cookie.into());
        self
    }

    pub fn set_cookies(mut self, cookies: impl Iterator<Item = impl Into<Cookie>>) -> Self {
        for cookie in cookies {
            self.cookies.push(cookie.into());
        }
        self
    }

    pub fn get_cookies(&self) -> impl Iterator<Item = &Cookie> {
        self.cookies.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}

#[cfg(test)]
mod test {
    use crate::cookies::Cookie;

    use super::CookieJar;

    #[test]
    fn test() {
        let _ = CookieJar::new()
            .set_cookie(
                Cookie::new("Test", "Value")
                    .set_domain("/")
                    .set_max_age(24 * 60 * 60),
            )
            .set_cookie(("Test2".to_string(), "Value".to_string()))
            .set_cookie(("Test3", "Value".to_string()));
    }
}
