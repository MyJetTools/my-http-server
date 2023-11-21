use my_http_server::macros::*;
use my_http_server::*;

use super::models::*;

#[http_route(
    method: "GET",
    route: "/api/testVecOfEnumAsI32",
    summary: "Test vec of enum as i32",
    description: "Test vec of enum as i32",
    controller: "TestVecOfEnumAsI32",
    input_data: "GetOrdersHttpInputData",
    result:[
        {status_code: 200, description: "Ok response"},
    ]
)]

pub struct TestVecOfEnumAsI32Action {}

impl TestVecOfEnumAsI32Action {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &TestVecOfEnumAsI32Action,
    input_data: GetOrdersHttpInputData,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    println!("InputData: {:?}", input_data);
    return HttpOutput::Empty.into_ok_result(true).into();
}
