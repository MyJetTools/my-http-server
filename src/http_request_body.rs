use std::collections::HashMap;

use rust_extensions::slice_of_u8_utils::SliceOfU8Ext;
use serde::de::DeserializeOwned;

use crate::{
    body_data_reader::BodyDataReader, HttpFailResult, JsonEncodedData, UrlEncodedData,
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
    pub content_type: Option<String>,
    raw_body: Vec<u8>,
    body_content_type: BodyContentType,
}

impl HttpRequestBody {
    pub fn new(body: Vec<u8>, content_type: Option<String>) -> Self {
        let body_content_type = BodyContentType::detect(body.as_slice());
        Self {
            raw_body: body,
            body_content_type,
            content_type,
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

    pub fn get_body_data_reader(&self) -> Result<BodyDataReader, HttpFailResult> {
        if let Some(content_type) = self.content_type.as_ref() {
            if extract_boundary(content_type.as_bytes()).is_some() {
                let reader = BodyDataReader::create_as_form_data(&self.raw_body);
                return Ok(reader);
            }
        }
        match self.body_content_type {
            BodyContentType::Json => get_body_data_reader_as_json_encoded(self.raw_body.as_slice()),
            BodyContentType::UrlEncoded => {
                let body_as_str = self.as_str()?;
                get_body_data_reader_as_url_encoded(body_as_str)
            }
            BodyContentType::Unknown => {
                return Err(HttpFailResult::as_not_supported_content_type(
                    "Unknown body content type".to_string(),
                ))
            }
        }
    }
}

fn get_body_data_reader_as_url_encoded(
    body_as_str: &str,
) -> Result<BodyDataReader, HttpFailResult> {
    match UrlEncodedData::from_body(body_as_str) {
        Ok(result) => return Ok(BodyDataReader::crate_as_url_encoded_data(result)),
        Err(err) => {
            let result = HttpFailResult {
                write_telemetry: true,
                content: format!("Can not parse Form Data. {:?}", err).into_bytes(),
                content_type: WebContentType::Text,
                status_code: 412,
                write_to_log: true,
                #[cfg(feature = "my-telemetry")]
                add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            };

            return Err(result);
        }
    }
}

impl<T: DeserializeOwned> TryInto<Vec<T>> for HttpRequestBody {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        crate::input_param_value::parse_json_value(self.as_slice())
    }
}

impl<T: DeserializeOwned> TryInto<HashMap<String, T>> for HttpRequestBody {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<HashMap<String, T>, Self::Error> {
        crate::input_param_value::parse_json_value(self.as_slice())
    }
}

fn get_body_data_reader_as_json_encoded(body: &[u8]) -> Result<BodyDataReader, HttpFailResult> {
    match JsonEncodedData::new(body) {
        Ok(result) => Ok(BodyDataReader::create_as_json_encoded_data(result)),
        Err(err) => {
            let result = HttpFailResult {
                write_telemetry: true,
                content: format!("Can not parse Form Data. {:?}", err).into_bytes(),
                content_type: WebContentType::Text,
                status_code: 412,
                write_to_log: true,
                #[cfg(feature = "my-telemetry")]
                add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            };

            return Err(result);
        }
    }
}

fn extract_boundary(src: &[u8]) -> Option<&[u8]> {
    let pos = src.find_sequence_pos("boundary".as_bytes(), 0)?;

    let pos = src.find_byte_pos('=' as u8, pos)?;

    let end_pos = src.find_byte_pos(';' as u8, pos);

    match end_pos {
        Some(end_pos) => Some(&src[pos + 1..end_pos]),
        None => Some(&src[pos + 1..]),
    }
}

impl TryInto<u8> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u8, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<u8>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse u8. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<i8> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i8, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<i8>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse u8. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<u16> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u16, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<u16>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse u16. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<i16> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i16, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<i16>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse u16. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<u32> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u32, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<u32>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse u32. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<i32> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i32, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<i32>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse u32. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<u64> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u64, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<u64>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse u64. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<i64> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i64, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<i64>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse i64. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<usize> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<usize, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<usize>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse usize. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<isize> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<isize, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<isize>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse isize. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<f32> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f32, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<f32>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse f32. {:?}",
                err
            ))),
        }
    }
}

impl TryInto<f64> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f64, Self::Error> {
        let str = self.as_str()?;

        match str.parse::<f64>() {
            Ok(result) => Ok(result),
            Err(err) => Err(HttpFailResult::as_validation_error(format!(
                "Can not parse f64. {:?}",
                err
            ))),
        }
    }
}

impl<'s> TryInto<&'s str> for &'s HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<&'s str, Self::Error> {
        Ok(self.as_str()?)
    }
}

impl TryInto<String> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(self.as_str()?.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let body = r#"{"processId":"8269e2ac-fa3b-419a-8e65-1a606ba07942","sellAmount":0.4,"buyAmount":null,"sellAsset":"ETH","buyAsset":"USDT"}"#;

        let body = HttpRequestBody::new(body.as_bytes().to_vec(), None);

        let form_data = body.get_body_data_reader().unwrap();

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

        let result = form_data.get_optional("buyAmount");
        assert!(result.is_none());
    }

    #[test]
    fn test_boundary_extractor() {
        let content_type_header =
            "multipart/form-data; boundary=----WebKitFormBoundaryXayIfSQWkEtJ6k10";

        let boundary = extract_boundary(content_type_header.as_bytes()).unwrap();

        assert_eq!(
            "----WebKitFormBoundaryXayIfSQWkEtJ6k10",
            std::str::from_utf8(boundary).unwrap()
        );
    }
}
