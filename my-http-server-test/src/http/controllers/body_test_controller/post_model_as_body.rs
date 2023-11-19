use my_http_server::macros::*;
use my_http_server::*;

use super::contracts::*;

#[http_route(
    method: "POST",
    route: "/api/postBodyAsModel",
    summary: "Test of body as model",
    description: "Test of body as model",
    controller: "TestBodyModel",
    input_data: "BodyAsModel",
    result:[
        {status_code: 202, description: "Ok response"},
    ]
)]
pub struct PostBodyAsModel;

async fn handle_request(
    _action: &PostBodyAsModel,
    _input_data: BodyAsModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Empty.into_ok_result(true).into();
}
