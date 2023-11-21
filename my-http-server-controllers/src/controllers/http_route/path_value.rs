pub struct PathValue<'s> {
    pub name: &'static str,
    pub value: &'s str,
}

impl<'s> PathValue<'s> {
    pub fn new(name: &'static str, value: &'s str) -> Self {
        Self { name, value }
    }

    pub fn as_str(&self) -> &str {
        self.value
    }
}
