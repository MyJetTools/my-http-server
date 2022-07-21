use serde::de::DeserializeOwned;

use crate::{
    form_data::FormData, HttpFailResult, JsonEncodedData, UrlEncodedData, UrlEncodedDataSource,
    WebContentType,
};

pub enum BodyContentType {
    Json,
    UrlEncoded,
    Unknown,
}

impl BodyContentType {
    pub fn detect(raw_body: &[u8]) -> Self {
        for b in raw_body {
            if *b <= 32 {
                continue;
            }

            if *b == '{' as u8 || *b == '[' as u8 {
                return BodyContentType::Json;
            } else {
                return BodyContentType::UrlEncoded;
            }
        }
        Self::Unknown
    }
}

pub struct HttpRequestBody {
    raw_body: Vec<u8>,
    body_content_type: BodyContentType,
}

impl HttpRequestBody {
    pub fn new(body: Vec<u8>) -> Self {
        let body_content_type = BodyContentType::detect(body.as_slice());
        Self {
            raw_body: body,
            body_content_type,
        }
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

    pub fn get_form_data(&self) -> Result<FormData, HttpFailResult> {
        match self.body_content_type {
            BodyContentType::Json => get_form_data_as_json_encoded(self.raw_body.as_slice()),
            BodyContentType::UrlEncoded => {
                let body_as_str = self.as_str()?;
                get_form_data_as_url_encoded(body_as_str)
            }
            BodyContentType::Unknown => {
                return Err(HttpFailResult::as_not_supported_content_type(
                    "Unknown body content type".to_string(),
                ))
            }
        }
    }
}

fn get_form_data_as_url_encoded(body_as_str: &str) -> Result<FormData, HttpFailResult> {
    match UrlEncodedData::new(body_as_str, UrlEncodedDataSource::FormData) {
        Ok(result) => return Ok(FormData::crate_as_url_encoded_data(result)),
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

fn get_form_data_as_json_encoded(body: &[u8]) -> Result<FormData, HttpFailResult> {
    match JsonEncodedData::new(body) {
        Ok(result) => Ok(FormData::create_as_json_encoded_data(result)),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let body = r#"{"processId":"8269e2ac-fa3b-419a-8e65-1a606ba07942","sellAmount":0.4,"buyAmount":null,"sellAsset":"ETH","buyAsset":"USDT"}"#;

        let body = HttpRequestBody::new(body.as_bytes().to_vec());

        let form_data = body.get_form_data().unwrap();

        assert_eq!(
            "8269e2ac-fa3b-419a-8e65-1a606ba07942",
            form_data
                .get_required("processId")
                .unwrap()
                .as_string()
                .unwrap()
        );

        assert_eq!(
            "0.4",
            form_data
                .get_required("sellAmount")
                .unwrap()
                .as_string()
                .unwrap()
        );
    }
}
