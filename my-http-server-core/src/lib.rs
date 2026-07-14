pub mod errors;
mod http_ctx;
mod http_fail_result;

mod http_ok_result;
mod http_path;

mod http_server_middleware;

mod request_credentials;
//mod request_flow;
mod request_ip;

mod http_server;
mod http_server_data;

mod web_content_type;

// ── The value / reader / conversion / field-type layer is owned by my-http-utils (the same lib
// fl-url and other clients use), so a `#[derive(MyHttpInput)]` model compiles on both sides.
// Core re-exports it under the historical paths; server-only glue (hyper body, headers, path,
// RequestReader) stays here.
pub use my_http_utils::form_data_reader::{FormDataItem, FormDataReader};
pub use my_http_utils::http_input::core::{
    data_src, extract_web_form_boundary, BodyContentType, BodyReader, JsonEncodedData,
    JsonEncodedValueAsString, QueryStringReader,
};
pub use my_http_utils::http_input::{
    FileContent, HttpInputValue, HttpParseError, PasswordHttpInputField, RawData, RawDataTyped,
};

pub use http_ctx::HttpContext;
pub use http_fail_result::HttpFailResult;
pub use http_ok_result::*;
pub use http_path::HttpPath;

pub use http_server::*;

pub use http_server_middleware::*;
pub use request_credentials::*;
//pub use request_flow::HttpServerRequestFlow;
pub use request_ip::*;
pub use web_content_type::WebContentType;

pub use http_server_data::*;

mod http_request;
pub use http_request::*;
mod http_headers;
pub use http_headers::*;
pub mod cookies;
mod http_output;
pub use http_output::*;

pub extern crate async_trait;
pub extern crate hyper;
pub extern crate my_hyper_utils;
