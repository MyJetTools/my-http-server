pub struct RootPaths {
    items: Vec<&'static str>,
}

impl RootPaths {
    pub fn add(&mut self, src: &'static str) {
        self.items.push(src);
    }

    pub fn is_my_path(&self, the_path: &str) -> bool {
        for path in self.items.iter() {
            if the_path.eq_ignore_ascii_case(*path) {
                return true;
            }
        }

        false
    }
}

impl Default for RootPaths {
    fn default() -> Self {
        Self { items: vec!["/"] }
    }
}
