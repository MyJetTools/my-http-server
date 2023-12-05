use my_http_server::macros::*;
use my_http_server::*;

use super::contracts::*;

#[http_route(
    method: POST,
    route: "/api/bodymodel/v1",
    summary: "Test of body as model",
    description: "Test of body as model",
    controller: "Test",
    authorized: "No",
    input_data: PostBodyModel,
    result:[
        {status_code: 200, description: "Ok response", model: DoneResult},
    ]
)]
pub struct PostAction {}

impl PostAction {
    pub fn new() -> Self {
        Self {}
    }
}

async fn handle_request(
    _action: &PostAction,
    input_data: PostBodyModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    println!("InputData: {:?}", input_data);
    return HttpOutput::Empty.into_ok_result(true).into();
}
