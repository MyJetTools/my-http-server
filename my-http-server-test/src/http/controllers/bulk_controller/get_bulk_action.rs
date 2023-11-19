use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rest_api_shared::authorisation::GetClientId;

use crate::app::AppContext;

use super::contracts::*;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/api/keyvalue/v1/Bulk/Get",
    summary: "Get values by keys",
    description: "Returns values by keys",
    controller: "Bulk",
    input_data: "GetKeysInputModel",

    result:[
        {status_code: 200, description: "Ok response", model: "GetKeyValuesResponse"},
    ]
)]
pub struct GetBulkAction {
    app: Arc<AppContext>,
}

impl GetBulkAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
async fn handle_request(
    action: &GetBulkAction,
    input_data: GetKeysInputModel,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let client_id = ctx.get_client_id()?;

    for key in &input_data.keys {
        crate::http::validators::validate_key(key)?;
    }

    let values = action
        .app
        .key_value_grpc_client
        .get_by_keys(&ctx.telemetry_context, client_id, input_data.keys)
        .await;

    let mut data = Vec::with_capacity(values.len());

    for value in values {
        data.push(KeyValueHttpModel {
            key: value.key,
            value: value.value,
        });
    }

    let result = GetKeyValuesResponse { data };

    return HttpOutput::as_json(result).into_ok_result(true).into();
}
