use std::{sync::Arc, time::Duration};

use my_http_server_core::HttpFailResult;

use crate::{MyWebSocketHttpRequest, WsMessage};

use super::MyWebSocket;

#[async_trait::async_trait]
pub trait MyWebSocketCallback {
    async fn connected(
        &self,
        my_web_socket: Arc<MyWebSocket>,
        http_request: MyWebSocketHttpRequest,
        disconnect_timeout: Duration,
    ) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, my_web_socket: &MyWebSocket);
    async fn on_message(&self, my_web_socket: Arc<MyWebSocket>, message: WsMessage);
}
