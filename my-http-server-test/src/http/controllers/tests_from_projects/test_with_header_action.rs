use my_http_server::macros::*;
use my_http_server::*;

#[derive(MyHttpInput)]
pub struct ProcessCallbackApiRequest {
    #[http_header(name = "x-signature", description = "Signature for request validation")]
    pub sign: Option<String>,
    #[http_header(
        name: "x-signature-with-default",
        description: "Signature for request validation",
        default: "33"
    )]
    pub sign_with_default: String,

    #[http_header(
        name = "x-signature-2",
        description = "Signature for request validation"
    )]
    pub sign2: String,

    #[http_body_raw(description = "body")]
    pub body: my_http_server::types::RawData,
}

#[http_route(
    method: "POST",
    route: "/testHeaders",
    description: "Gets list of trending assets",
    summary: "Gets list of trending assets",
    controller: "assets",
    input_data: "ProcessCallbackApiRequest",

    result: [
        {status_code: 200, description: "Ok response"},
    ]
)]
pub struct TestWithHeaderAction;

async fn handle_request(
    _action: &TestWithHeaderAction,
    request: ProcessCallbackApiRequest,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = format!("request: {:?}", request.sign);
    return HttpOutput::as_text(result).into_ok_result(false);
}
