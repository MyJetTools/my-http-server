use crate::{data_src::SRC_HEADER, HttpFailResult};

/// A raw header value. Typed reading of headers into a model is done by my-http-utils via
/// [`my_http_utils::http_input::core::THttpRequest`] (`RequestReader` surfaces headers to it), so
/// this stays a thin accessor — only `as_str`, used by the server's own host/cookie/scheme/ip helpers.
pub struct HeaderValue<'s> {
    pub name: &'static str,
    pub value: &'s [u8],
}

impl<'s> HeaderValue<'s> {
    pub fn new(name: &'static str, value: &'s [u8]) -> Self {
        Self { name, value }
    }

    pub fn from_header_value(name: &'static str, value: &'s hyper::header::HeaderValue) -> Self {
        Self {
            name,
            value: value.as_bytes(),
        }
    }

    pub fn as_str(&self) -> Result<&'s str, HttpFailResult> {
        match std::str::from_utf8(self.value) {
            Ok(result) => Ok(result),
            Err(_) => Err(HttpFailResult::invalid_value_to_parse(format!(
                "Can not parse header value in {}",
                SRC_HEADER
            ))),
        }
    }
}
