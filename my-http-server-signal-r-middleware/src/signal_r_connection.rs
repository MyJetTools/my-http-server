use std::{
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
};

use my_http_server_web_sockets::MyWebSocket;
use rust_extensions::{
    date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds},
    TaskCompletion,
};

use tokio::sync::Mutex;

use tokio_tungstenite::tungstenite::Message;

use crate::{SignalRConnectionId, SignalRConnectionToken, SignalRParam};

pub struct MySignalRConnectionSingleThreaded {
    web_socket: Option<Arc<MyWebSocket>>,
    long_pooling: Option<TaskCompletion<String, String>>,
}

pub struct MySignalRConnection<TCtx: Send + Sync + 'static> {
    single_threaded: Mutex<MySignalRConnectionSingleThreaded>,
    pub connection_id: SignalRConnectionId,
    pub connection_token: Option<SignalRConnectionToken>,
    pub created: DateTimeAsMicroseconds,
    pub last_incoming_moment: AtomicDateTimeAsMicroseconds,
    connected: AtomicBool,
    has_greeting: AtomicBool,
    pub negotiation_version: usize,
    pub ctx: TCtx,
}

impl<TCtx: Send + Sync + Default + 'static> MySignalRConnection<TCtx> {
    pub fn new(
        connection_id: SignalRConnectionId,
        connection_token: Option<SignalRConnectionToken>,
        negotiation_version: usize,
        web_socket: Option<Arc<MyWebSocket>>,
    ) -> Self {
        Self {
            single_threaded: Mutex::new(MySignalRConnectionSingleThreaded {
                web_socket,
                long_pooling: None,
            }),
            connection_id,
            connection_token,
            negotiation_version,
            created: DateTimeAsMicroseconds::now(),
            last_incoming_moment: AtomicDateTimeAsMicroseconds::now(),
            connected: AtomicBool::new(true),
            has_greeting: AtomicBool::new(false),
            ctx: TCtx::default(),
        }
    }

    pub fn get_list_index(&self) -> &String {
        if let Some(token) = self.connection_token.as_ref() {
            token.as_ref_of_string()
        } else {
            self.connection_id.as_ref_of_string()
        }
    }

    pub fn get_has_greeting(&self) -> bool {
        self.has_greeting.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_has_greeting(&self) {
        self.has_greeting
            .store(true, std::sync::atomic::Ordering::SeqCst)
    }

    pub async fn has_web_socket(&self, web_socket_id: i64) -> bool {
        let read_access = self.single_threaded.lock().await;
        if let Some(web_socket) = &read_access.web_socket {
            return web_socket.id == web_socket_id;
        }

        false
    }

    pub async fn get_web_socket(&self) -> Option<Arc<MyWebSocket>> {
        let read_access = self.single_threaded.lock().await;
        read_access.web_socket.clone()
    }

    pub fn update_incoming_activity(&self) {
        let now = DateTimeAsMicroseconds::now();
        self.last_incoming_moment.update(now);
    }

    pub fn get_last_incoming(&self) -> DateTimeAsMicroseconds {
        self.last_incoming_moment.as_date_time()
    }

    pub async fn send<'s>(&self, action_name: &str, parameter: &SignalRParam<'s>) {
        let web_socket = {
            let read_access = self.single_threaded.lock().await;
            read_access.web_socket.clone()
        };

        if let Some(web_socket) = web_socket {
            let mut result = String::new();

            result.push_str("{\"type\":1,\"target\":\"");
            result.push_str(action_name);
            result.push_str("\",\"arguments\":[");
            match parameter {
                SignalRParam::JsonObject(json_writer) => {
                    json_writer.build_into(&mut result);
                }
                SignalRParam::String(value) => {
                    result.push('"');
                    my_json::json_string_value::write_escaped_json_string_value(value, &mut result);
                    result.push('"');
                }
                SignalRParam::Number(number) => {
                    result.push_str(number.to_string().as_str());
                }
                SignalRParam::Float(value) => {
                    result.push_str(value.to_string().as_str());
                }
                SignalRParam::Boolean(value) => {
                    if *value {
                        result.push_str("true");
                    } else {
                        result.push_str("false");
                    }
                }
                SignalRParam::Raw(value) => {
                    for (index, item) in value.iter().enumerate() {
                        if index > 0 {
                            result.push(',');
                        }

                        unsafe {
                            result.push_str(std::str::from_utf8_unchecked(item));
                        }
                    }
                }
                SignalRParam::None => {}
            }

            result.push_str("]}");
            result.push(30 as char);

            web_socket
                .send_message([Message::Text(result.into())].into_iter())
                .await;
        }
    }

    pub async fn send_ping_payload(&self) {
        self.send_raw_payload(crate::messages::get_ping_payload().to_string())
            .await;
    }

    pub async fn send_raw_payload(&self, mut raw_payload: String) {
        let web_socket = {
            let read_access = self.single_threaded.lock().await;
            read_access.web_socket.clone()
        };

        raw_payload.push(30 as char);

        if let Some(web_socket) = web_socket {
            web_socket
                .send_message([Message::Text(raw_payload.into())].into_iter())
                .await;
        }
    }

    pub async fn assign_web_socket(&self, web_socket: Arc<MyWebSocket>) {
        let new_id = web_socket.id;
        let mut write_access = self.single_threaded.lock().await;

        if let Some(old_websocket) = write_access.web_socket.replace(web_socket) {
            old_websocket
                .send_message(
                    [Message::Text(
                        format!(
                            "SignalR WebSocket {} has been kicked by Websocket {} ",
                            old_websocket.id, new_id
                        )
                        .into(),
                    )]
                    .into_iter(),
                )
                .await;
        }
    }

    pub async fn disconnect(&self) -> Option<Arc<MyWebSocket>> {
        let mut write_access = self.single_threaded.lock().await;

        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let mut result = None;

        if let Some(web_socket) = write_access.web_socket.take() {
            web_socket.disconnect().await;
            result = Some(web_socket);
        }

        if let Some(mut long_pooling) = write_access.long_pooling.take() {
            long_pooling.set_error(format!("Canceling this LongPool since we disconnect it."));
        }

        result
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub async fn get_addr(&self) -> Option<SocketAddr> {
        let read_access = self.single_threaded.lock().await;
        if let Some(web_socket) = &read_access.web_socket {
            return Some(web_socket.addr);
        }
        None
    }
}
