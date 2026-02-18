use my_http_server_core::HttpPath;

pub struct FilesMapping {
    pub uri_prefix: HttpPath,
    pub folder_path: String,
}

impl FilesMapping {
    pub fn new(uri_prefix: &str, folder_path: &str) -> Self {
        Self {
            uri_prefix: HttpPath::from_str(uri_prefix),
            folder_path: folder_path.to_string(),
        }
    }

    pub fn to_lowercase(&mut self) {
        self.folder_path = self.folder_path.to_lowercase();
    }
}
