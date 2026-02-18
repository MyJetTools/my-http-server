use my_http_server_core::HttpPath;
use rust_extensions::str_utils::StrUtils;

#[derive(Default)]
pub struct NoCache {
    no_cache: Vec<&'static str>,
}

impl NoCache {
    pub fn add_path(&mut self, path: &'static str) {
        self.no_cache.push(path);
    }

    pub fn marked_as_no_cache(&mut self, path: &HttpPath) -> bool {
        for itm in self.no_cache.iter() {
            if path.as_str().eq_case_insensitive(itm) {
                return true;
            }
        }

        false
    }
}
