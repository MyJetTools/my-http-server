use crate::{HttpFailResult, UrlEncodedData};

#[derive(Clone)]
pub enum BodyContentType {
    Json,
    UrlEncoded,
    FormData(String),
    Unknown,
    Empty,
}

impl BodyContentType {
    pub fn is_unknown_or_empty(&self) -> bool {
        match self {
            BodyContentType::Json => false,
            BodyContentType::UrlEncoded => false,
            BodyContentType::FormData(_) => false,
            BodyContentType::Unknown => true,
            BodyContentType::Empty => true,
        }
    }

    pub fn from_content_type(content_type: &str) -> Result<Self, HttpFailResult> {
        if content_type.len() == 0 {
            return Ok(Self::Unknown);
        }

        let content_type_lower_case = content_type.to_lowercase();
        if content_type_lower_case.contains("json") {
            return Ok(Self::Json);
        }

        if content_type_lower_case.contains("x-www-form-urlencoded") {
            return Ok(Self::UrlEncoded);
        }

        if content_type_lower_case.contains("multipart/form-data") {
            let boundary = extract_web_form_boundary(content_type);

            match boundary {
                Some(boundary_src) => {
                    return Ok(Self::FormData(boundary_src.to_string()));
                }
                None => {
                    return Err(HttpFailResult::as_fatal_error(format!(
                        "Can not extract FromData boundary from content type '{}'",
                        content_type
                    )));
                }
            }
        }

        Ok(Self::Unknown)
    }

    pub fn detect_from_body(raw_body: &[u8]) -> Option<Self> {
        for b in raw_body {
            if *b <= 32 {
                continue;
            }

            if *b == '{' as u8 || *b == '[' as u8 {
                return Some(Self::Json);
            } else {
                break;
            }
        }

        if let Ok(body_as_str) = std::str::from_utf8(raw_body) {
            if body_as_str.contains('=') && UrlEncodedData::from_body(body_as_str).is_ok() {
                return Some(Self::UrlEncoded);
            }
        }

        None
    }
}

pub fn extract_web_form_boundary(content_type: &str) -> Option<&str> {
    for itm in content_type.split(";") {
        let itm = itm.trim();
        if itm.starts_with("boundary=") {
            let boundary = itm.trim_start_matches("boundary=");
            return Some(boundary);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::extract_web_form_boundary;

    #[test]
    fn test_boundary_extractor() {
        let content_type_header =
            "multipart/form-data; boundary=----WebKitFormBoundaryXayIfSQWkEtJ6k10";

        let boundary = extract_web_form_boundary(content_type_header).unwrap();

        assert_eq!("----WebKitFormBoundaryXayIfSQWkEtJ6k10", boundary);
    }
}
