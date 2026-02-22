use std::collections::HashMap;

use serde::de::DeserializeOwned;
use url_utils::server::FormDataReader;

use crate::{data_src::*, *};

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
            Ok(result) => {
                return Ok(result);
            }
            Err(err) => return Err(HttpFailResult::as_fatal_error(format!("{}", err))),
        }
    }

    pub fn get_body_data_reader<'s>(&'s self) -> Result<BodyDataReader<'s>, HttpFailResult> {
        match &self.body_content_type {
            BodyContentType::Json => get_body_data_reader_as_json_encoded(self.raw_body.as_slice()),
            BodyContentType::UrlEncoded => {
                let body_as_str = self.as_str()?;
                get_body_data_reader_as_url_encoded(body_as_str)
            }

            BodyContentType::FormData(boundary) => {
                let form_data_reader = FormDataReader::new(&self.raw_body, boundary.as_str());
                Ok(BodyDataReader::create_as_form_data_reader(form_data_reader))
            }

            BodyContentType::Unknown => Ok(BodyDataReader::Unknown(self.raw_body.as_slice())),
            BodyContentType::Empty => Ok(BodyDataReader::Empty),
        }
    }
}

fn get_body_data_reader_as_url_encoded<'s>(
    body_as_str: &'s str,
) -> Result<BodyDataReader<'s>, HttpFailResult> {
    match UrlEncodedData::from_body(body_as_str) {
        Ok(result) => return Ok(BodyDataReader::crate_as_url_encoded_data(result)),
        Err(err) => {
            let output = HttpOutput::Content {
                status_code: 412,
                headers: Default::default(),
                content: format!("Can not parse Url Encoded Data. {:?}", err).into_bytes(),
            };

            return Err(HttpFailResult::new(output, true, true));
        }
    }
}

impl<T: DeserializeOwned> TryInto<Vec<T>> for HttpRequestBodyContent {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        crate::convert_from_str::to_json("RawBody".into(), &Some(self.as_slice()), "HttpBody")
    }
}

impl<T: DeserializeOwned> TryInto<HashMap<String, T>> for HttpRequestBodyContent {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<HashMap<String, T>, Self::Error> {
        crate::convert_from_str::to_json("RawBody", &Some(self.as_slice()), "Body")
    }
}

fn get_body_data_reader_as_json_encoded<'s>(
    body: &'s [u8],
) -> Result<BodyDataReader<'s>, HttpFailResult> {
    match JsonEncodedData::new(body) {
        Ok(result) => Ok(BodyDataReader::create_as_json_encoded_data(result)),
        Err(err) => {
            let output = HttpOutput::Content {
                status_code: 412,
                headers: Default::default(),
                content: format!("Can not parse Json Encoded Data. {:?}", err).into_bytes(),
            };

            return Err(HttpFailResult::new(output, true, true));
        }
    }
}

impl TryInto<u8> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u8, Self::Error> {
        let str = self.as_str()?;

        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<i8> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i8, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<u16> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u16, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<i16> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i16, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<u32> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u32, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<i32> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i32, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<u64> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u64, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<i64> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i64, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<usize> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<usize, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<isize> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<isize, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<f32> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f32, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<f64> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f64, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl<'s> TryInto<&'s str> for &'s HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<&'s str, Self::Error> {
        Ok(self.as_str()?)
    }
}

impl TryInto<String> for HttpRequestBodyContent {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(self.as_str()?.to_string())
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
    fn test_json_encoded_with_unknown_content_type() {
        let body = r#"{"processId":"8269e2ac-fa3b-419a-8e65-1a606ba07942","sellAmount":0.4,"buyAmount":null,"sellAsset":"ETH","buyAsset":"USDT"}"#;

        let body = HttpRequestBodyContent::new(body.as_bytes().to_vec(), BodyContentType::Unknown)
            .unwrap();

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
    fn test_url_encoded() {
        let body = r#"product=test&name=test2&yaml=test"#;

        let body =
            HttpRequestBodyContent::new(body.as_bytes().to_vec(), BodyContentType::UrlEncoded)
                .unwrap();

        let form_data = body.get_body_data_reader().unwrap();

        assert_eq!(
            "test",
            form_data
                .get_required("product")
                .unwrap()
                .as_string()
                .unwrap()
        );

        assert_eq!(
            "test2",
            form_data.get_required("name").unwrap().as_string().unwrap()
        );
    }

    #[test]
    fn test_url_encoded_with_unknown_content_type() {
        let body = r#"product=test&name=test2&yaml=test"#;

        let body = HttpRequestBodyContent::new(body.as_bytes().to_vec(), BodyContentType::Unknown)
            .unwrap();

        let form_data = body.get_body_data_reader().unwrap();

        assert_eq!(
            "test",
            form_data
                .get_required("product")
                .unwrap()
                .as_string()
                .unwrap()
        );

        assert_eq!(
            "test2",
            form_data.get_required("name").unwrap().as_string().unwrap()
        );
    }
}
