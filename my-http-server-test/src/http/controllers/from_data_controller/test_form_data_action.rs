use my_http_server::macros::*;
use my_http_server::*;

use super::contracts::*;

#[http_route(
    method: "POST",
    route: "/api/formdata/v1",
    deprecated_routes: ["/api/formdata/v1/deprecated"],
    summary: "Test of body as model",
    description: "Test of body as model",
    controller: "FormDataTest",
    input_data: "FromDataInputModel",
    result:[
        {status_code: 202, description: "Ok response"},
    ]
)]

pub struct TestFormDataAction {}

impl TestFormDataAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &TestFormDataAction,
    input_data: FromDataInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    println!("InputData: {:?}", input_data.dd);
    return HttpOutput::Empty.into_ok_result(true).into();
}
