use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use rest_api_shared::authorisation::GetClientId;

use crate::app::AppContext;

use super::contracts::*;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/api/keyvalue/v1/Bulk/Delete",
    summary: "Delete keys in bulk",
    description: "Deletes keys in bulk",
    controller: "Bulk",
    input_data: "DeleteKeysInputModel",

    result:[
        {status_code: 202, description: "Ok response"},
    ]
)]
pub struct DeleteBulkAction {
    app: Arc<AppContext>,
}

impl DeleteBulkAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}
async fn handle_request(
    action: &DeleteBulkAction,
    input_data: DeleteKeysInputModel,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let client_id = ctx.get_client_id()?;

    action
        .app
        .key_value_grpc_client
        .delete_by_keys(&ctx.telemetry_context, client_id, input_data.keys)
        .await;

    return HttpOutput::Empty.into_ok_result(true).into();
}
