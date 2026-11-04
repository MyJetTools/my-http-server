#[derive(Debug, Clone)]
pub enum WebContentType {
    Html,
    Css,
    Png,
    Svg,
    JavaScript,
    Json,
    Text,
    Yaml,
    Wasm,
    Protobuf,
    Raw(String),
}

impl WebContentType {
    pub fn as_str(&self) -> &str {
        match self {
            WebContentType::Html => "text/html; charset=utf-8",
            WebContentType::Css => "text/css; charset=utf-8",
            WebContentType::JavaScript => "text/javascript; charset=utf-8",
            // No charset: application/json has no such parameter - JSON is UTF-8 by definition.
            WebContentType::Json => "application/json",
            WebContentType::Text => "text/plain; charset=utf-8",
            WebContentType::Png => "image/png",
            WebContentType::Svg => "image/svg+xml",
            WebContentType::Yaml => "text/yaml",
            WebContentType::Wasm => "application/wasm",
            WebContentType::Protobuf => "application/x-protobuf",
            WebContentType::Raw(content_type) => content_type.as_str(),
        }
    }

    /// The content type of a file by the extension of its name - in any letter case. `path` may
    /// be a bare file name or a path with `/` or `\` separators; only the last segment is looked
    /// at, so a dot in a folder name does not count. `None` for a name without an extension, and
    /// for an extension not known here.
    ///
    /// Types without a variant of their own come back as [`WebContentType::Raw`].
    pub fn detect_by_extension(path: &str) -> Option<Self> {
        let file_name = match path.rfind(['/', '\\']) {
            Some(index) => &path[index + 1..],
            None => path,
        };

        let (_, extension) = file_name.rsplit_once('.')?;

        // No extension known here is that long - and a long one is not worth lowercasing.
        if extension.is_empty() || extension.len() > 16 {
            return None;
        }

        let result = match extension.to_ascii_lowercase().as_str() {
            "html" | "htm" => WebContentType::Html,
            "css" => WebContentType::Css,
            "js" | "mjs" => WebContentType::JavaScript,
            "json" | "map" => WebContentType::Json,
            "txt" | "text" => WebContentType::Text,
            "yaml" | "yml" => WebContentType::Yaml,
            "wasm" => WebContentType::Wasm,
            "png" => WebContentType::Png,
            "svg" => WebContentType::Svg,
            "webmanifest" => raw("application/manifest+json"),
            "xml" => raw("application/xml"),
            "csv" => raw("text/csv; charset=utf-8"),
            "pdf" => raw("application/pdf"),
            "ico" => raw("image/x-icon"),
            "jpg" | "jpeg" => raw("image/jpeg"),
            "gif" => raw("image/gif"),
            "webp" => raw("image/webp"),
            "avif" => raw("image/avif"),
            "woff" => raw("font/woff"),
            "woff2" => raw("font/woff2"),
            "ttf" => raw("font/ttf"),
            "otf" => raw("font/otf"),
            "mp4" => raw("video/mp4"),
            "webm" => raw("video/webm"),
            "mp3" => raw("audio/mpeg"),
            "ogg" => raw("audio/ogg"),
            "wav" => raw("audio/wav"),
            _ => return None,
        };

        Some(result)
    }
}

fn raw(content_type: &str) -> WebContentType {
    WebContentType::Raw(content_type.to_string())
}

impl Into<WebContentType> for String {
    fn into(self) -> WebContentType {
        WebContentType::Raw(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(path: &str) -> Option<String> {
        WebContentType::detect_by_extension(path).map(|ct| ct.as_str().to_string())
    }

    #[test]
    fn known_extensions() {
        for (path, expected) in [
            ("index.html", "text/html; charset=utf-8"),
            ("index.htm", "text/html; charset=utf-8"),
            ("site.css", "text/css; charset=utf-8"),
            ("app.js", "text/javascript; charset=utf-8"),
            ("app.mjs", "text/javascript; charset=utf-8"),
            ("data.json", "application/json"),
            ("app.js.map", "application/json"),
            ("readme.txt", "text/plain; charset=utf-8"),
            ("readme.text", "text/plain; charset=utf-8"),
            ("config.yaml", "text/yaml"),
            ("config.yml", "text/yaml"),
            ("app.wasm", "application/wasm"),
            ("logo.png", "image/png"),
            ("logo.svg", "image/svg+xml"),
            ("site.webmanifest", "application/manifest+json"),
            ("sitemap.xml", "application/xml"),
            ("report.csv", "text/csv; charset=utf-8"),
            ("report.pdf", "application/pdf"),
            ("favicon.ico", "image/x-icon"),
            ("photo.jpg", "image/jpeg"),
            ("photo.jpeg", "image/jpeg"),
            ("anim.gif", "image/gif"),
            ("photo.webp", "image/webp"),
            ("photo.avif", "image/avif"),
            ("font.woff", "font/woff"),
            ("font.woff2", "font/woff2"),
            ("font.ttf", "font/ttf"),
            ("font.otf", "font/otf"),
            ("clip.mp4", "video/mp4"),
            ("clip.webm", "video/webm"),
            ("sound.mp3", "audio/mpeg"),
            ("sound.ogg", "audio/ogg"),
            ("sound.wav", "audio/wav"),
        ] {
            assert_eq!(detect(path).as_deref(), Some(expected), "{path}");
        }
    }

    #[test]
    fn the_letter_case_of_the_extension_does_not_matter() {
        assert_eq!(
            detect("APP.JS").as_deref(),
            Some("text/javascript; charset=utf-8")
        );
        assert_eq!(
            detect("Worker.Mjs").as_deref(),
            Some("text/javascript; charset=utf-8")
        );
        assert_eq!(detect("Photo.JPEG").as_deref(), Some("image/jpeg"));
        assert_eq!(detect("/fonts/Inter.WOFF2").as_deref(), Some("font/woff2"));
    }

    #[test]
    fn paths_are_accepted() {
        assert_eq!(
            detect("/assets/app.js").as_deref(),
            Some("text/javascript; charset=utf-8")
        );
        assert_eq!(
            detect("./wwwroot/index.html").as_deref(),
            Some("text/html; charset=utf-8")
        );
        assert_eq!(detect("C:\\www\\logo.png").as_deref(), Some("image/png"));
    }

    #[test]
    fn a_name_without_an_extension_has_no_type() {
        for path in ["", "/", "README", "/api/json", "/js", "file.", "/folder/"] {
            assert_eq!(detect(path), None, "{path}");
        }
    }

    #[test]
    fn a_dot_in_a_folder_name_is_not_an_extension() {
        assert_eq!(detect("/v1.json/data"), None);
        assert_eq!(detect("/assets.v2/README"), None);
        assert_eq!(detect("C:\\site.css\\file"), None);
        assert_eq!(
            detect("/assets.v2/app.js").as_deref(),
            Some("text/javascript; charset=utf-8")
        );
    }

    #[test]
    fn an_unknown_extension_has_no_type() {
        assert_eq!(detect("archive.xyz"), None);
        assert_eq!(detect("archive.verylongextensionname"), None);
    }

    #[test]
    fn text_types_carry_the_charset_and_json_does_not() {
        assert_eq!(WebContentType::Html.as_str(), "text/html; charset=utf-8");
        assert_eq!(WebContentType::Css.as_str(), "text/css; charset=utf-8");
        assert_eq!(
            WebContentType::JavaScript.as_str(),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(WebContentType::Text.as_str(), "text/plain; charset=utf-8");
        assert_eq!(WebContentType::Json.as_str(), "application/json");
    }
}
