// End-to-end coverage of the my-http-utils `parse` the server now relies on: every input source
// (path / query / header / JSON body), a query default, an absent optional, a ctx-less validator,
// and the `HttpParseError` -> `HttpFailResult` status mapping. Parsing is driven purely by the
// `MyHttpInput` derive — there is no server-side model-description macro anymore.

use my_http_server::macros::*;
use my_http_server::HttpFailResult;
use my_http_utils::http_input::core::THttpRequest;
use my_http_utils::http_input::HttpParseError;

// New unified, ctx-less validator contract: `fn(&str) -> Result<(), impl ToString>`.
fn not_empty(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err("must not be empty".to_string())
    } else {
        Ok(())
    }
}

#[derive(MyHttpInput, Debug)]
pub struct AllSourcesRequest {
    #[http_path(name = "id", description = "id")]
    pub id: i32,

    #[http_query(name = "notify", description = "notify")]
    pub notify: bool,

    #[http_query(name = "limit", description = "limit", default = 10)]
    pub limit: u32,

    #[http_query(name = "tag", description = "tag")]
    pub tag: Option<String>,

    #[http_header(name = "X-Token", description = "token", validator = "not_empty")]
    pub token: String,

    #[http_body(name = "email", description = "email")]
    pub email: String,
}

/// In-memory `THttpRequest` — the same abstraction the server's `RequestReader` implements over a
/// real hyper request.
struct MockRequest {
    query: &'static str,
    headers: Vec<(&'static str, &'static str)>,
    path: Vec<(&'static str, &'static str)>,
    body: &'static [u8],
    content_type: Option<&'static str>,
}

impl THttpRequest for MockRequest {
    fn get_query_string(&self) -> &str {
        self.query
    }
    fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| *v)
    }
    fn get_path_value(&self, name: &str) -> Option<&str> {
        self.path.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
    }
    fn get_body(&self) -> &[u8] {
        self.body
    }
    fn get_content_type(&self) -> Option<&str> {
        self.content_type
    }
}

fn base_request() -> MockRequest {
    MockRequest {
        query: "notify=true",
        headers: vec![("x-token", "secret")],
        path: vec![("id", "42")],
        body: br#"{"email":"a@b.com"}"#,
        content_type: Some("application/json"),
    }
}

#[test]
fn parses_every_source_with_default_and_absent_optional() {
    let parsed = AllSourcesRequest::parse(&base_request()).unwrap();

    assert_eq!(parsed.id, 42); // path
    assert!(parsed.notify); // query bool
    assert_eq!(parsed.limit, 10); // query default (absent in request)
    assert_eq!(parsed.tag, None); // query optional (absent)
    assert_eq!(parsed.token, "secret"); // header, matched case-insensitively
    assert_eq!(parsed.email, "a@b.com"); // json body

    assert!(AllSourcesRequest::READS_BODY);
}

#[test]
fn query_overrides_default_and_reads_optional() {
    let mut req = base_request();
    req.query = "notify=false&limit=5&tag=vip";

    let parsed = AllSourcesRequest::parse(&req).unwrap();
    assert!(!parsed.notify);
    assert_eq!(parsed.limit, 5);
    assert_eq!(parsed.tag, Some("vip".to_string()));
}

#[test]
fn missing_required_header_maps_to_400() {
    let mut req = base_request();
    req.headers = vec![];

    let err = AllSourcesRequest::parse(&req).unwrap_err();
    assert!(matches!(
        err,
        HttpParseError::RequiredParameterIsMissing { .. }
    ));

    let fail: HttpFailResult = err.into();
    assert_eq!(fail.output.get_status_code(), 400);
}

#[test]
fn unparsable_bool_maps_to_400() {
    let mut req = base_request();
    req.query = "notify=maybe";

    let err = AllSourcesRequest::parse(&req).unwrap_err();
    assert!(matches!(err, HttpParseError::CanNotParseValue { .. }));

    let fail: HttpFailResult = err.into();
    assert_eq!(fail.output.get_status_code(), 400);
}

#[test]
fn validator_rejection_maps_to_validation_400() {
    let mut req = base_request();
    req.headers = vec![("x-token", "   ")];

    let err = AllSourcesRequest::parse(&req).unwrap_err();
    assert!(matches!(err, HttpParseError::Validation(_)));

    let fail: HttpFailResult = err.into();
    assert_eq!(fail.output.get_status_code(), 400);
}
