use my_http_server::macros::*;
use my_http_server::*;

#[derive(Debug, MyHttpInput)]
pub struct GetChartHttpInputData {
    #[http_path(name: "AccountId", description:"Account Id")]
    pub account_id: String,
}

#[http_route(
    method: "GET",
    route: "/api/Dashboard/v1/Chart/{AccountId}",
    summary: "Get trader's account chart",
    input_data: "GetChartHttpInputData",
    description: "Get trader's account chart",
    controller: "Dashboard",
    result:[
        {status_code: 200, description: "Ok response"},
    ]
)]
pub struct TestPath;

async fn handle_request(
    _action: &TestPath,
    _input_data: GetChartHttpInputData,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::Empty.into_ok_result(true).into();
}
