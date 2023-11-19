use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rest_api_shared::authorisation::GetClientId;

use crate::app::AppContext;

use super::contracts::*;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/api/keyvalue/v1/Bulk/Post",
    summary: "Update keys/values",
    description: "Updates values by keys",
    controller: "Bulk",
    input_data: "PostKeysInputModel",
    authorized: ["WithdrawalTwoFaConfirmed"],

    result:[
        {status_code: 202, description: "Ok response"},
    ]
)]
pub struct PostBulkAction {
    app: Arc<AppContext>,
}

impl PostBulkAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
async fn handle_request(
    action: &PostBulkAction,
    input_data: PostKeysInputModel,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let client_id = ctx.get_client_id()?;

    let mut input = Vec::new();

    for kv in input_data.values {
        crate::http::validators::validate_key(&kv.key)?;

        input.push(crate::repositories::KeyValue {
            key: kv.key,
            value: kv.value,
        });
    }

    action
        .app
        .key_value_grpc_client
        .add_by_keys(&ctx.telemetry_context, client_id, input)
        .await;

    return HttpOutput::Empty.into_ok_result(true).into();
}
