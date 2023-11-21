use my_http_server::macros::*;
use my_http_server::*;

#[http_route(
    method: "GET",
    route: "/api/testAuth",
    summary: "Test of body as model",
    description: "Test of body as model"
    controller: "TestPath",
    authorized: "No",
    result:[
        {status_code: 200, description: "Ok response"},
    ]
)]
pub struct TestAuthYes;

async fn handle_request(
    _action: &TestAuthYes,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Empty.into_ok_result(true).into();
}
