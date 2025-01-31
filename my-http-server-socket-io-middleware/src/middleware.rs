use std::{sync::Arc, time::Duration};

use hyper::Method;
use my_http_server_core::*;
use rust_extensions::Logger;
use socket_io_utils::SocketIoSettings;
use tokio::sync::Mutex;

use crate::{
    namespaces::SocketIoNameSpaces, MySocketIo, MySocketIoCallbacks, SocketIoList,
    WebSocketCallbacks,
};

pub struct MySocketIoEngineMiddleware {
    pub path_prefix: String,
    socket_id: Mutex<i64>,
    web_socket_callback: Arc<WebSocketCallbacks>,
    socket_io_list: Arc<SocketIoList>,
    registered_sockets: Arc<SocketIoNameSpaces>,
    connections_callback: Arc<dyn MySocketIoCallbacks + Send + Sync + 'static>,
    pub settings: Arc<SocketIoSettings>,
    disconnect_timeout: Duration,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl MySocketIoEngineMiddleware {
    pub fn new(
        connections_callback: Arc<dyn MySocketIoCallbacks + Send + Sync + 'static>,
        settings: Arc<SocketIoSettings>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        let registered_sockets = Arc::new(SocketIoNameSpaces::new());
        let socket_io_list = Arc::new(SocketIoList::new());

        Self {
            socket_io_list: socket_io_list.clone(),

            path_prefix: "/socket.io/".to_string(),
            web_socket_callback: Arc::new(WebSocketCallbacks {
                socket_io_list,
                registered_sockets: registered_sockets.clone(),
                connections_callback: connections_callback.clone(),
                settings: settings.clone(),
            }),
            socket_id: Mutex::new(0),
            registered_sockets,
            connections_callback,
            settings,
            disconnect_timeout: Duration::from_secs(60),
            logger,
        }
    }

    pub async fn register_socket_io(&self, socket_io: Arc<dyn MySocketIo + Send + Sync + 'static>) {
        self.registered_sockets.add(socket_io).await;
    }

    async fn get_socket_id(&self) -> i64 {
        let mut socket_no = self.socket_id.lock().await;
        *socket_no += 1;
        *socket_no
    }
}

#[async_trait::async_trait]
impl HttpServerMiddleware for MySocketIoEngineMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
    ) -> Option<Result<HttpOkResult, HttpFailResult>> {
        if !rust_extensions::str_utils::compare_strings_case_insensitive(
            ctx.request.http_path.as_str(),
            self.path_prefix.as_str(),
        ) {
            return None;
        }

        if ctx
            .request
            .get_headers()
            .try_get_case_insensitive("sec-websocket-key")
            .is_some()
        {
            let id = self.get_socket_id().await;
            let result = my_http_server_web_sockets::handle_web_socket_upgrade(
                &mut ctx.request,
                self.web_socket_callback.clone(),
                id,
                self.disconnect_timeout,
                self.logger.clone(),
            )
            .await;

            return Some(result);

            /*
            if let RequestData::AsRaw(request) = &mut ctx.request.req {
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
             */
        }

        match ctx.request.method {
            Method::GET => {
                let result = handle_get_request(
                    ctx,
                    &self.connections_callback,
                    &self.socket_io_list,
                    &self.settings,
                )
                .await;
                return Some(result);
            }
            Method::POST => {
                return Some(handle_post_request(ctx));
            }
            _ => None,
        }
    }
}

async fn handle_get_request(
    ctx: &mut HttpContext,
    connections_callback: &Arc<dyn MySocketIoCallbacks + Send + Sync + 'static>,
    socket_io_list: &Arc<SocketIoList>,
    settings: &Arc<SocketIoSettings>,
) -> Result<HttpOkResult, HttpFailResult> {
    let query = ctx.request.get_query_string();

    let query = match query {
        Ok(query) => query,
        Err(err) => {
            return HttpFailResult::as_fatal_error(format!("{:?}", err)).into_err();
        }
    };

    let sid = query.get_optional("sid");

    if let Some(sid) = sid {
        let sid = sid.as_str()?;

        let response = socket_io_utils::SocketIoHandshakeOpenModel::from_settings(
            sid.to_string(),
            settings.as_ref(),
        );
        return HttpOutput::as_json(response).into_ok_result(false);
    } else {
        let (_, model) =
            crate::process_connect(connections_callback, socket_io_list, settings, None).await;

        return HttpOutput::as_json(model).into_ok_result(false);
    }
}

fn handle_post_request(_ctx: &mut HttpContext) -> Result<HttpOkResult, HttpFailResult> {
    return HttpOutput::as_text("ok").into_ok_result(true).into();
}
