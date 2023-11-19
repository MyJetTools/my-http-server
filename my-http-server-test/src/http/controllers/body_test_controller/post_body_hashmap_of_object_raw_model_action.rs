use my_http_server::macros::*;
use my_http_server::*;

use super::contracts::*;

#[http_route(
    method: "POST",
    route: "/api/bodyHashmapOfObjectRawModel",
    summary: "Test of body model",
    description: "Test of body model",
    controller: "TestBodyModel",
    input_data: "BodyModelHashmapOfObjectRawHttpInput",
    result:[
        {status_code: 202, description: "Ok response"},
    ]
)]
pub struct PostBodyHashmapOfObjectRawModelAction {}

impl PostBodyHashmapOfObjectRawModelAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &PostBodyHashmapOfObjectRawModelAction,
    input_data: BodyModelHashmapOfObjectRawHttpInput,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Empty.into_ok_result(true).into();
}
