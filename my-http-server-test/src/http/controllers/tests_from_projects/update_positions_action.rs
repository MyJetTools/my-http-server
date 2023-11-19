use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput, Debug)]
pub struct UpdatePositionApiRequest {
    #[http_path(name = "id"; description = "Id of position")]
    pub position_id: String,
    #[http_body(name = "WalletId"; description = "Id of wallet")]
    pub wallet_id: String,
    #[http_body(name = "DesirePrice"; description = "")]
    pub desire_price: Option<f64>,
    #[http_body(name = "TopUpEnabled"; description = "")]
    pub top_up_enabled: Option<bool>,
    #[http_body(name = "StopLoss"; description = "")]
    pub stop_loss: Option<UpdateAutoCloseConfigApiModel>,
    #[http_body(name = "TakeProfit"; description = "")]
    pub take_profit: Option<UpdateAutoCloseConfigApiModel>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpInputObjectStructure)]
pub struct UpdateAutoCloseConfigApiModel {
    #[serde(rename = "Value")]
    pub value: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct UpdatePositionApiResponse {
    #[serde(rename = "data")]
    pub data: String,
    #[serde(rename = "result")]
    pub response_code: i32,
    #[serde(rename = "message")]
    pub message: String,
}

#[http_route(
    method: "PUT",
    route: "/trading-rest-api/v1/execution/positions/{id}",
    description: "Updates order params of opened position",
    controller: "positions",
    input_data: "UpdatePositionApiRequest",
    summary: "",

    result:[
        {status_code: 200, description: "Ok response"},
    ]
)]
pub struct UpdatePositionAction;

async fn handle_request(
    _action: &UpdatePositionAction,
    input_data: UpdatePositionApiRequest,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    println!("Input data: {:?}", input_data);

    return HttpOutput::Empty.into_ok_result(true);
}
