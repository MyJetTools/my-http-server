use std::{
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};

use my_http_server_core::{
    HttpContext, HttpFailResult, HttpOkResult, HttpRequestHeaders, HttpServerMiddleware,
};
use rust_extensions::{Logger, StrOrString};

use crate::MyWebSocketCallback;

pub struct MyWebsocketMiddleware<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>
{
    path: StrOrString<'static>,
    callbacks: Arc<TMyWebSocketCallback>,
    socket_id: AtomicI64,
    disconnect_timeout: Duration,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>
    MyWebsocketMiddleware<TMyWebSocketCallback>
{
    pub fn new(
        path: StrOrString<'static>,
        callbacks: Arc<TMyWebSocketCallback>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        Self {
            path,
            callbacks,
            socket_id: AtomicI64::new(0),
            disconnect_timeout: Duration::from_secs(60),
            logger,
        }
    }

    fn get_socket_id(&self) -> i64 {
        self.socket_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static> HttpServerMiddleware
    for MyWebsocketMiddleware<TMyWebSocketCallback>
{
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        if !ctx
            .request
            .get_path()
            .starts_with_case_insensitive(self.path.as_str())
        {
            return None;
        }

        if ctx
            .request
            .get_headers()
            .try_get_case_insensitive("sec-websocket-key")
            .is_none()
        {
            return None;
        }

        let id = self.get_socket_id();
        let result = crate::helpers::handle_web_socket_upgrade(
            &mut ctx.request,
            self.callbacks.clone(),
            id,
            self.disconnect_timeout,
            self.logger.clone(),
        )
        .await;

        Some(result)
    }
}
