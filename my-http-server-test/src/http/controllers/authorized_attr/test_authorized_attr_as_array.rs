use my_http_server::macros::*;
use my_http_server::*;

#[http_route(
    method: "GET",
    route: "/api/authorized",
    summary: "Test with Authorized attribute",
    description: "Test with Authorized attribute",
    controller: "TestVecOfEnumAsI32",
    authorized: ["Admin"],
    result:[
        {status_code: 200, description: "Ok response"},
    ]
)]

pub struct TestAuthorizedAction {}

impl TestAuthorizedAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &TestAuthorizedAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Empty.into_ok_result(true).into();
}
