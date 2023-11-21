use my_http_server::macros::*;
use my_http_server::*;

use super::contracts::*;

#[http_route(
    method: "GET",
    route: "/api/testPath/{pathKey}",
    summary: "Test of body as model",
    description: "Test of body as model",
    controller: "TestPath",
    input_data: "TestPathModel",
    result:[
        {status_code: 200, description: "Ok response"},
    ]
)]
pub struct TestPathAction {}

impl TestPathAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &TestPathAction,
    input_data: TestPathModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    println!("InputData: {:?}", input_data);
    return HttpOutput::Empty.into_ok_result(true).into();
}
