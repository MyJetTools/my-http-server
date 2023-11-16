use std::sync::Arc;

use hyper::Method;
use my_http_server_core::{
    HttpContext, HttpFailResult, HttpOkResult, HttpOutput, HttpPath, HttpServerMiddleware,
    HttpServerRequestFlow, WebContentType,
};
use rust_extensions::Logger;
use tokio::sync::Mutex;

use crate::{
    my_signal_r_actions::MySignalRActions, MiddlewareBuilder, SignalRConnectionsList,
    WebSocketCallbacks,
};

pub struct MySignalRMiddleware<TCtx: Send + Sync + Default + 'static> {
    pub hub_name: String,
    negotiate_path: HttpPath,
    socket_id: Mutex<i64>,
    web_socket_callback: Arc<WebSocketCallbacks<TCtx>>,
    signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
    actions: Arc<MySignalRActions<TCtx>>,
    disconnect_timeout: std::time::Duration,
}

impl<TCtx: Send + Sync + Default + 'static> MySignalRMiddleware<TCtx> {
    pub fn new_with_builder(
        hub_name: &str,
        signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> MiddlewareBuilder<TCtx> {
        MiddlewareBuilder::new(hub_name.to_string(), signal_r_list, logger)
    }

    pub fn new(
        hub_name: &str,
        signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
        actions: MySignalRActions<TCtx>,
        disconnect_timeout: std::time::Duration,
    ) -> Self {
        let hub_name = hub_name.to_lowercase();

        let actions = Arc::new(actions);

        Self {
            negotiate_path: compile_negotiate_uri(hub_name.as_str()),
            signal_r_list: signal_r_list.clone(),
            hub_name,
            web_socket_callback: Arc::new(WebSocketCallbacks {
                signal_r_list,
                my_signal_r_callbacks: actions.clone(),
            }),
            socket_id: Mutex::new(0),
            actions,
            disconnect_timeout,
        }
    }

    async fn get_socket_id(&self) -> i64 {
        let mut socket_no = self.socket_id.lock().await;
        *socket_no += 1;
        *socket_no
    }

    async fn handle_negotiate_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Result<HttpOkResult, HttpFailResult> {
        #[cfg(feature = "debug_ws")]
        println!("handle_negotiate_request");
        let query_string_result = ctx.request.get_query_string();

        let negotiation_version = match query_string_result {
            Ok(value) => {
                if let Some(result) = value.get_optional("negotiateVersion") {
                    result.value.parse::<usize>().unwrap()
                } else {
                    0
                }
            }
            Err(_) => 0,
        };

        let (_, response) = crate::process_connect(
            &self.actions,
            &self.signal_r_list,
            negotiation_version,
            None,
        )
        .await;
        HttpOutput::Content {
            headers: None,
            content_type: Some(WebContentType::Text),
            content: response.into_bytes(),
        }
        .into_ok_result(true)
        .into()
    }
}

#[async_trait::async_trait]
impl<TCtx: Send + Sync + Default + 'static> HttpServerMiddleware for MySignalRMiddleware<TCtx> {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        let http_path = ctx.request.get_http_path();
        if !http_path.has_value_at_index_case_insensitive(0, &self.hub_name) {
            return get_next.next(ctx).await;
        }

        if ctx
            .request
            .get_optional_header("sec-websocket-key")
            .is_some()
        {
            if let Some(request) = &mut ctx.request.try_unwrap_raw_request() {
                let id = self.get_socket_id().await;
                return my_http_server_web_sockets::handle_web_socket_upgrade(
                    request,
                    self.web_socket_callback.clone(),
                    id,
                    ctx.request.addr,
                    self.disconnect_timeout,
                )
                .await;
            }

            return get_next.next(ctx).await;
        }

        if ctx.request.get_method() == Method::POST {
            if ctx
                .request
                .get_http_path()
                .is_the_same_to(&self.negotiate_path)
            {
                return self.handle_negotiate_request(ctx).await;
            }
        }

        get_next.next(ctx).await
    }
}

fn compile_negotiate_uri(hub_name: &str) -> HttpPath {
    let mut result = String::new();

    if !hub_name.starts_with('/') {
        result.push('/');
    }

    result.push_str(hub_name);

    if !hub_name.ends_with('/') {
        result.push('/');
    }

    result.push_str("negotiate");

    HttpPath::from_string(result)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_negotiate_compilation() {
        let name = "/signalr";

        let result = super::compile_negotiate_uri(name);

        assert!(result.has_values_at_index_case_insensitive(0, &["signalr", "negotiate"]));
    }
}
