use rust_extensions::MaybeShortString;

use crate::HttpFailResult;

pub enum BodyContentType {
    Json,
    UrlEncoded,
    FormData(MaybeShortString),
    Unknown,
    Empty,
}

impl BodyContentType {
    pub fn detect(raw_body: &[u8], content_type: Option<&String>) -> Result<Self, HttpFailResult> {
        if raw_body.is_empty() {
            return Ok(Self::Empty);
        }

        if let Some(content_type) = content_type {
            let lower_case = MaybeShortString::from_str(content_type);
            let lower_case = lower_case.as_str();
            if lower_case.contains("multipart/form-data") {
                let boundary = extract_web_form_boundary(content_type);

                match boundary {
                    Some(boundary_src) => {
                        return Ok(Self::FormData(MaybeShortString::from_str(boundary_src)));
                    }
                    None => {
                        return Err(HttpFailResult::as_fatal_error(format!(
                            "Can not extract FromData boundary from content type '{}'",
                            content_type
                        )));
                    }
                }
            }

            if lower_case.contains("json") {
                return Ok(Self::Json);
            }

            if lower_case.contains("x-www-form-urlencoded") {
                return Ok(Self::UrlEncoded);
            }

            return Err(HttpFailResult::as_fatal_error(format!(
                "Content type '{}' is not supported",
                content_type
            )));
        }

        for b in raw_body {
            if *b <= 32 {
                continue;
            }

            if *b == '{' as u8 || *b == '[' as u8 {
                return Ok(Self::Json);
            } else {
                break;
            }
        }

        return Ok(Self::Unknown);
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
