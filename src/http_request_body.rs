use serde::de::DeserializeOwned;

use crate::{HttpFailResult, QueryString, QueryStringDataSource, WebContentType};

pub struct HttpRequestBody {
    raw_body: Vec<u8>,
}

impl HttpRequestBody {
    pub fn new(body: Vec<u8>) -> Self {
        Self { raw_body: body }
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
            Ok(result) => {
                return Ok(result);
            }
            Err(err) => return Err(HttpFailResult::as_fatal_error(format!("{}", err))),
        }
    }

    pub fn get_form_data(&self) -> Result<QueryString, HttpFailResult> {
        let body_as_str = self.as_str()?;

        match QueryString::new(body_as_str, QueryStringDataSource::FormData) {
            Ok(result) => return Ok(result),
            Err(err) => {
                let result = HttpFailResult {
                    write_telemetry: true,
                    content: format!("Can not parse Form Data. {:?}", err).into_bytes(),
                    content_type: WebContentType::Text,
                    status_code: 412,
                };

                return Err(result);
            }
        }
    }
}
