use super::models::*;
use my_http_server::macros::*;
use my_http_server::*;

#[http_route(
    method: "GET",
    route: "/api/testResultWithGeneric2",
    summary: "Test result with Generic 2",
    description: "Test result with Generic 2",
    controller: "TestResultWithGeneric",
    result:[
        {status_code: 200, description: "Ok response", model: "ResponseWithResult<MyData2>"},
    ]
)]
pub struct TestResultWithGeneric2;

async fn handle_request(
    _action: &TestResultWithGeneric2,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Empty.into_ok_result(true).into();
}
