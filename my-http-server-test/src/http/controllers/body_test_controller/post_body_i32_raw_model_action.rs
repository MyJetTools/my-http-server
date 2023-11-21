use my_http_server::macros::*;
use my_http_server::*;

use super::contracts::*;

#[http_route(
    method: "POST",
    route: "/api/bodyI32RawModel",
    summary: "Test of body model",
    description: "Test of body model",
    controller: "TestBodyModel",
    input_data: "BodyModelI32RawHttpInput",
    result:[
        {status_code: 202, description: "Ok response"},
    ]
)]
pub struct PostBodyI32RawModelAction {}

impl PostBodyI32RawModelAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &PostBodyI32RawModelAction,
    input_data: BodyModelI32RawHttpInput,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::as_json(input_data.body)
        .into_ok_result(true)
        .into();
}
