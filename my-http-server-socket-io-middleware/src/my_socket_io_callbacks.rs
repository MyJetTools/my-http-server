use std::sync::Arc;

use my_http_server_core::*;

use crate::MySocketIoConnection;

#[async_trait::async_trait]
pub trait MySocketIoCallbacks {
    async fn connected(&self, socket_io: Arc<MySocketIoConnection>) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, socket_io: Arc<MySocketIoConnection>);

    async fn on_callback(
        &self,
        socket_io: &Arc<MySocketIoConnection>,
        ns: &str,
        event_name: &str,
        data: &str,
    ) -> Option<String>;
}
