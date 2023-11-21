use my_http_server::macros::*;
use my_http_server::*;

use super::contracts::*;

#[http_route(
    method: "POST",
    route: "/api/bodystringrawmodel",
    summary: "Test of body model",
    description: "Test of body model",
    controller: "TestBodyModel",
    input_data: "BodyModelStringRawHttpInput",
    result:[
        {status_code: 202, description: "Ok response"},
    ]
)]
pub struct PostBodyStringRawModelAction {}

impl PostBodyStringRawModelAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &PostBodyStringRawModelAction,
    input_data: BodyModelStringRawHttpInput,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::as_json(input_data.body)
        .into_ok_result(true)
        .into();
}
