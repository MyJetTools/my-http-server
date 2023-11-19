use super::models::*;
use my_http_server::macros::*;
use my_http_server::*;

#[http_route(
    method: "GET",
    route: "/api/testResultWithGeneric",
    summary: "Test result with Generic",
    description: "Test result with Generic",
    controller: "TestResultWithGeneric",
    result:[
        {status_code: 200, description: "Ok response", model: "ResponseWithResult<MyData>"},
    ]
)]

pub struct TestResultWithGeneric;

async fn handle_request(
    _action: &TestResultWithGeneric,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Empty.into_ok_result(true).into();
}
