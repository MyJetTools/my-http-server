use std::{sync::Arc, time::Duration};

use crate::{MyWebSocketHttpRequest, WsMessage};

use super::MyWebSocket;

#[derive(Debug)]
pub struct WebSocketConnectedFail {
    pub reason: String,
    pub write_to_logs: bool,
}

#[async_trait::async_trait]
pub trait MyWebSocketCallback {
    async fn connected(
        &self,
        my_web_socket: Arc<MyWebSocket>,
        http_request: MyWebSocketHttpRequest,
        disconnect_timeout: Duration,
    ) -> Result<(), WebSocketConnectedFail>;
    async fn disconnected(&self, my_web_socket: &MyWebSocket);
    async fn on_message(&self, my_web_socket: Arc<MyWebSocket>, message: WsMessage);
}
