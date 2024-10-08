use std::{sync::Arc, time::Duration};

use my_http_server_core::HttpRequest;
use my_http_server_core::{HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use rust_extensions::Logger;

use crate::MyWebSocketCallback;

pub async fn handle_web_socket_upgrade<
    TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static,
>(
    req: &mut HttpRequest,
    callback: Arc<TMyWebSocketCallback>,
    id: i64,
    disconnect_timeout: Duration,
    logs: Arc<dyn Logger + Send + Sync + 'static>,
) -> Result<HttpOkResult, HttpFailResult> {
    let query_string = if let Some(query_string) = req.get_uri().query() {
        Some(query_string.to_string())
    } else {
        None
    };

    let addr = req.addr.clone();

    let req = req.take_incoming_body();

    let upgrade_result = crate::web_sockets_upgrade::upgrade(
        id,
        addr,
        query_string,
        req,
        callback.clone(),
        disconnect_timeout,
        logs,
    )
    .await;

    match upgrade_result {
        Ok(response) => {
            return HttpOutput::Raw(response).into_ok_result(true);
        }
        Err(err) => {
            let content = format!("Can not upgrade websocket. Reason: {}", err);
            println!("{}", content);
            return Err(HttpFailResult::new(
                WebContentType::Text,
                400,
                content.into_bytes(),
                false,
                false,
            ));
        }
    }
}
