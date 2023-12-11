use my_http_server::macros::*;
use my_http_server::*;

use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct GetTrendingAssetsApiRequest {
    #[http_query(name: "limitOpt"; description: "Limit for records count. Default is 5", default:5)]
    pub limit_op: Option<i32>,

    #[http_query(name: "limit"; description: "Limit for records count", default:7)]
    pub limit: i32,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct GetTrendingAssetsApiResponse {
    #[serde(rename = "data")]
    pub data: Vec<TrendingAssetApiModel>,
    #[serde(rename = "result")]
    pub response_code: i32,
    #[serde(rename = "message")]
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TrendingAssetApiModel {
    #[serde(rename = "symbol")]
    pub symbol: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    #[serde(rename = "lastPrice")]
    pub last_price: f64,
    #[serde(rename = "priceChangePercent24h")]
    pub price_change_percent_24h: f64,
}

#[http_route(
    method: "GET",
    route: "/statistics-rest-api/v1/assets/trending",
    description: "Gets list of trending assets",
    summary: "Gets list of trending assets",
    controller: "assets",
    input_data: "GetTrendingAssetsApiRequest",

    result: [
        {status_code: 200, description: "Ok response", model: "GetTrendingAssetsApiResponse"},
    ]
)]

pub struct GetPositionsAction;

async fn handle_request(
    action: &GetPositionsAction,
    request: GetTrendingAssetsApiRequest,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    todo!("Implement")
}
