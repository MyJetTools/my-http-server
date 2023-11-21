use super::contracts::*;
use my_http_server::macros::*;
use my_http_server::*;

#[http_route(
    method: "POST",
    route: "/api/bodyVecOfStringRawModel",
    summary: "Test of body model",
    description: "Test of body model",
    controller: "TestBodyModel",
    input_data: "BodyModelVecOfStringRawHttpInput",
    result:[
        {status_code: 202, description: "Ok response"},
    ]
)]
pub struct PostBodyVecOfStringRawModelAction {}

impl PostBodyVecOfStringRawModelAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &PostBodyVecOfStringRawModelAction,
    input_data: BodyModelVecOfStringRawHttpInput,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::as_json(input_data.body)
        .into_ok_result(true)
        .into();
}
