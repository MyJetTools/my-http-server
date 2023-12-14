use std::collections::HashMap;

use serde::de::DeserializeOwned;

use crate::{
    data_src::*, form_data_reader::FormDataReader, BodyContentType, BodyDataReader, HttpFailResult,
    JsonEncodedData, UrlEncodedData, WebContentType,
};

pub struct HttpRequestBody {
    raw_body: Vec<u8>,
    body_content_type: BodyContentType,
}

impl HttpRequestBody {
    pub fn new(body: Vec<u8>, content_type: Option<String>) -> Result<Self, HttpFailResult> {
        let body_content_type = BodyContentType::detect(body.as_slice(), content_type.as_ref())?;
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

    pub fn get_body_data_reader(&self) -> Result<BodyDataReader, HttpFailResult> {
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
                #[cfg(feature = "with-telemetry")]
                add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            };

            return Err(result);
        }
    }
}

impl<T: DeserializeOwned> TryInto<Vec<T>> for HttpRequestBody {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        crate::convert_from_str::to_json("RawBody".into(), &Some(self.as_slice()), "HttpBody")
    }
}

impl<T: DeserializeOwned> TryInto<HashMap<String, T>> for HttpRequestBody {
    type Error = HttpFailResult;

    fn try_into(self) -> Result<HashMap<String, T>, Self::Error> {
        crate::convert_from_str::to_json("RawBody", &Some(self.as_slice()), "Body")
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
                #[cfg(feature = "with-telemetry")]
                add_telemetry_tags: my_telemetry::TelemetryEventTagsBuilder::new(),
            };

            return Err(result);
        }
    }
}

impl TryInto<u8> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u8, Self::Error> {
        let str = self.as_str()?;

        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<i8> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i8, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<u16> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u16, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<i16> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i16, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<u32> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u32, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<i32> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i32, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<u64> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<u64, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<i64> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<i64, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<usize> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<usize, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<isize> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<isize, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<f32> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f32, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
    }
}

impl TryInto<f64> for HttpRequestBody {
    type Error = HttpFailResult;
    fn try_into(self) -> Result<f64, Self::Error> {
        let str = self.as_str()?;
        crate::convert_from_str::to_simple_value("HttpBody", str, SRC_BODY)
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

        let body = HttpRequestBody::new(body.as_bytes().to_vec(), None).unwrap();

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
}
