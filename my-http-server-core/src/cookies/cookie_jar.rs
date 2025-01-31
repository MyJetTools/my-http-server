use super::*;

pub struct CookieJar {
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: Vec::new(),
        }
    }

    pub fn set_cookie<'s>(&mut self, cookie: impl Into<Cookie>) {
        self.cookies.push(cookie.into());
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
        let mut cookies = CookieJar::new();

        cookies.set_cookie(
            Cookie::new("Test", "Value")
                .set_domain("/")
                .set_max_age(24 * 60 * 60),
        );

        cookies.set_cookie(("Test2".to_string(), "Value".to_string()));

        cookies.set_cookie(("Test3", "Value".to_string()));
    }
}
