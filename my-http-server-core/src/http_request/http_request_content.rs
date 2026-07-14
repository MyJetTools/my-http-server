use serde::de::DeserializeOwned;

use crate::{BodyContentType, BodyReader, HttpFailResult};

/// The request body once received over hyper: raw bytes plus the detected content type. Typed
/// field reading is delegated to my-http-utils' [`BodyReader`] (same value layer the model
/// `parse` uses via `THttpRequest`), so this holds no conversions of its own.
pub struct HttpRequestBodyContent {
    raw_body: Vec<u8>,
    body_content_type: BodyContentType,
}

impl HttpRequestBodyContent {
    pub fn new(body: Vec<u8>, body_content_type: BodyContentType) -> Result<Self, HttpFailResult> {
        let body_content_type = if body_content_type.is_unknown_or_empty() {
            match BodyContentType::detect_from_body(body.as_slice()) {
                Some(body_type) => body_type,
                None => body_content_type,
            }
        } else {
            body_content_type
        };

        Ok(Self {
            raw_body: body,
            body_content_type,
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.raw_body
    }

    pub fn get_body(self) -> Vec<u8> {
        self.raw_body
    }

    pub fn as_str(&self) -> Result<&str, HttpFailResult> {
        match std::str::from_utf8(self.as_slice()) {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_fatal_error(format!(
                "Can not convert body to str. Err: {:?}",
                err
            ))),
        }
    }

    pub fn get_body_as_json<T>(&self) -> Result<T, HttpFailResult>
    where
        T: DeserializeOwned,
    {
        match serde_json::from_slice(self.as_slice()) {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_fatal_error(format!("{}", err))),
        }
    }

    /// Reads named body fields (JSON / url-encoded / multipart) via the shared my-http-utils
    /// [`BodyReader`] — the same value layer used by the derive-generated `parse`.
    pub fn get_body_data_reader<'s>(&'s self) -> Result<BodyReader<'s>, HttpFailResult> {
        let content_type = self.content_type_header();
        Ok(BodyReader::from_parts(
            self.raw_body.as_slice(),
            content_type.as_deref(),
        )?)
    }

    /// A representative `Content-Type` string rebuilt from the detected [`BodyContentType`], used
    /// to drive the my-http-utils body reader.
    fn content_type_header(&self) -> Option<String> {
        match &self.body_content_type {
            BodyContentType::Json => Some("application/json".to_string()),
            BodyContentType::UrlEncoded => Some("application/x-www-form-urlencoded".to_string()),
            BodyContentType::FormData(boundary) => {
                Some(format!("multipart/form-data; boundary={}", boundary))
            }
            BodyContentType::Unknown | BodyContentType::Empty => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_json_encoded() {
        let body = r#"{"processId":"8269e2ac-fa3b-419a-8e65-1a606ba07942","sellAmount":0.4,"buyAmount":null,"sellAsset":"ETH","buyAsset":"USDT"}"#;

        let body =
            HttpRequestBodyContent::new(body.as_bytes().to_vec(), BodyContentType::Json).unwrap();

        let reader = body.get_body_data_reader().unwrap();

        assert_eq!(
            "8269e2ac-fa3b-419a-8e65-1a606ba07942",
            reader.get_required("processId").unwrap().as_string().unwrap()
        );

        assert!(reader.get_optional("buyAmount").is_none());
    }

    #[test]
    fn test_url_encoded() {
        let body = r#"product=test&name=test2&yaml=test"#;

        let body =
            HttpRequestBodyContent::new(body.as_bytes().to_vec(), BodyContentType::UrlEncoded)
                .unwrap();

        let reader = body.get_body_data_reader().unwrap();

        assert_eq!(
            "test",
            reader.get_required("product").unwrap().as_string().unwrap()
        );
        assert_eq!(
            "test2",
            reader.get_required("name").unwrap().as_string().unwrap()
        );
    }
}
