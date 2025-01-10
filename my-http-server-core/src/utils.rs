use http_body_util::BodyExt;
use hyper::StatusCode;
use rust_extensions::StrOrString;

pub fn compile_response<'s>(
    status: StatusCode,
    text: impl Into<StrOrString<'s>>,
) -> crate::MyHttpServerResponse {
    let text = text.into();
    let full_body = http_body_util::Full::new(hyper::body::Bytes::from(text.to_string()));
    let builder = hyper::Response::builder().status(status);

    builder
        .body(full_body.map_err(|itm| itm.to_string()).boxed())
        .unwrap()
}

pub fn build_response(
    builder: http::response::Builder,
    body: Vec<u8>,
) -> crate::MyHttpServerResponse {
    let full_body = http_body_util::Full::new(hyper::body::Bytes::from(body));

    builder
        .body(full_body.map_err(|itm| itm.to_string()).boxed())
        .unwrap()
}

pub fn from_full_body(
    full_body: http::Response<http_body_util::Full<hyper::body::Bytes>>,
) -> crate::MyHttpServerResponse {
    let versions = full_body.version();
    let status = full_body.status();
    let (parts, full_body) = full_body.into_parts();

    let mut builder = hyper::Response::builder().status(status).version(versions);

    for header in parts.headers {
        if let Some(header_name) = header.0 {
            builder = builder.header(header_name, header.1);
        }
    }

    builder
        .body(full_body.map_err(|itm| itm.to_string()).boxed())
        .unwrap()
}
