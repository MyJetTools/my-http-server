use my_http_server_core::HttpPath;

/// Full set of headers which forbids any caching of the content
/// (browser, proxy or CDN).
pub const NO_CACHE_CACHE_CONTROL: &str = "no-store, no-cache, must-revalidate, max-age=0";
pub const NO_CACHE_PRAGMA: &str = "no-cache";
pub const NO_CACHE_EXPIRES: &str = "0";

#[derive(Default)]
pub struct NoCache {
    no_cache: Vec<HttpPath>,
}

impl NoCache {
    pub fn add_path(&mut self, path: String) {
        self.no_cache.push(HttpPath::from_string(compile_path(path)));
    }

    /// Path is compared segment by segment (case-insensitive, trailing slash
    /// does not matter). Query string is not a part of [HttpPath] - so any
    /// query string is matched.
    pub fn marked_as_no_cache(&self, path: &HttpPath) -> bool {
        self.no_cache.iter().any(|itm| itm.is_the_same_to(path))
    }
}

fn compile_path(mut path: String) -> String {
    if let Some(index) = path.find(|c| c == '?' || c == '#') {
        path.truncate(index);
    }

    if path.starts_with('/') {
        return path;
    }

    format!("/{}", path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create(paths: &[&str]) -> NoCache {
        let mut result = NoCache::default();
        for path in paths {
            result.add_path(path.to_string());
        }
        result
    }

    #[test]
    fn test_root_path() {
        let no_cache = create(&["/"]);

        assert!(no_cache.marked_as_no_cache(&HttpPath::from_str("/")));
        assert!(!no_cache.marked_as_no_cache(&HttpPath::from_str("/index.html")));
    }

    #[test]
    fn test_case_insensitive_and_trailing_slash() {
        let no_cache = create(&["/Index.html"]);

        assert!(no_cache.marked_as_no_cache(&HttpPath::from_str("/index.HTML")));

        let no_cache = create(&["/my/path"]);
        assert!(no_cache.marked_as_no_cache(&HttpPath::from_str("/my/path/")));
        assert!(!no_cache.marked_as_no_cache(&HttpPath::from_str("/my/path/sub")));
    }

    #[test]
    fn test_path_without_leading_slash() {
        let no_cache = create(&["index.html"]);
        assert!(no_cache.marked_as_no_cache(&HttpPath::from_str("/index.html")));
    }

    #[test]
    fn test_query_string_is_cut_off_from_registered_path() {
        let no_cache = create(&["/?a=b"]);
        assert!(no_cache.marked_as_no_cache(&HttpPath::from_str("/")));
    }

    #[test]
    fn test_no_paths_registered() {
        let no_cache = NoCache::default();
        assert!(!no_cache.marked_as_no_cache(&HttpPath::from_str("/")));
    }
}
