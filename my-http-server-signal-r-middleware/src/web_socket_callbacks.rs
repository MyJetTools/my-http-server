use std::{sync::Arc, time::Duration};

use my_http_server_core::HttpFailResult;
use my_http_server_web_sockets::{MyWebSocket, WebSocketMessage};
use my_json::json_reader::JsonFirstLineReader;
#[cfg(feature = "with-telemetry")]
use my_telemetry::MyTelemetryContext;
#[cfg(feature = "with-telemetry")]
use my_telemetry::TelemetryEventTagsBuilder;
use rust_extensions::array_of_bytes_iterator::SliceIterator;
use tokio_tungstenite::tungstenite::Message;

use crate::{
    messages::SignalRMessage, MySignalRCallbacks, MySignalRConnection, SignalRConnectionsList,
};

pub struct WebSocketCallbacks<TCtx: Send + Sync + Default + 'static> {
    pub signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
    pub my_signal_r_callbacks: Arc<dyn MySignalRCallbacks<TCtx = TCtx> + Send + Sync + 'static>,
}

#[async_trait::async_trait]
impl<TCtx: Send + Sync + Default + 'static> my_http_server_web_sockets::MyWebSocketCallback
    for WebSocketCallbacks<TCtx>
{
    async fn connected(
        &self,
        my_web_socket: Arc<MyWebSocket>,
        disconnect_timeout: Duration,
    ) -> Result<(), HttpFailResult> {
        #[cfg(feature = "debug_ws")]
        println!("connected web_socket:{}", my_web_socket.id);

        if let Some(query_string) = my_web_socket.get_query_string() {
            let connection_token = query_string.get_optional("id");

            if connection_token.is_none() {
                my_web_socket
                    .send_message(Message::Text("id query parameter is missing".to_string()))
                    .await;
                return Ok(());
            }

            let connection_token = connection_token.unwrap();

            match self
                .signal_r_list
                .assign_web_socket(
                    connection_token.get_raw_str().unwrap(),
                    my_web_socket.clone(),
                )
                .await
            {
                Some(signal_r_connection) => {
                    tokio::spawn(super::signal_r_liveness_loop::start(
                        self.my_signal_r_callbacks.clone(),
                        self.signal_r_list.clone(),
                        signal_r_connection,
                        disconnect_timeout,
                    ));
                }
                None => {
                    my_web_socket
                        .send_message(Message::Text(format!(
                            "SignalR with connection_token {} is not found",
                            connection_token.get_raw_str().unwrap(),
                        )))
                        .await;

                    return Ok(());
                }
            };
        }

        Ok(())
    }

    async fn disconnected(&self, my_web_socket: &MyWebSocket) {
        #[cfg(feature = "debug_ws")]
        println!("disconnected web_socket:{}", my_web_socket.id);
        let find_result = self
            .signal_r_list
            .get_by_web_socket_id(my_web_socket.id)
            .await;

        if let Some(signal_r_connection) = find_result {
            crate::process_disconnect(
                &self.signal_r_list,
                &signal_r_connection,
                self.my_signal_r_callbacks.clone(),
            )
            .await;
        }
    }
    async fn on_message(&self, my_web_socket: Arc<MyWebSocket>, message: WebSocketMessage) {
        #[cfg(feature = "debug_ws")]
        println!("Websocket{}, MSG: {:?}", my_web_socket.id, message);

        let signal_r = self
            .signal_r_list
            .get_by_web_socket_id(my_web_socket.id)
            .await;

        if let Some(signal_r_connection) = signal_r.as_ref() {
            signal_r_connection.update_incoming_activity();

            if let WebSocketMessage::String(value) = &message {
                if signal_r_connection.get_has_greeting() {
                    let mut json_first_line_iterator: JsonFirstLineReader<SliceIterator> =
                        value.as_bytes().into();
                    let packet_type = get_payload_type(&mut json_first_line_iterator);

                    if packet_type == "1" {
                        #[cfg(feature = "with-telemetry")]
                        let ctx = MyTelemetryContext::new();

                        #[cfg(feature = "with-telemetry")]
                        let started = rust_extensions::date_time::DateTimeAsMicroseconds::now();

                        let mut json_first_line_iterator: JsonFirstLineReader<SliceIterator> =
                            value.as_bytes().into();
                        let message = SignalRMessage::parse(&mut json_first_line_iterator);

                        #[cfg(feature = "with-telemetry")]
                        let ctx_spawned = ctx.clone();

                        let signal_r_callbacks = self.my_signal_r_callbacks.clone();

                        let connection_spawned = signal_r_connection.clone();

                        let target = message.target.to_string();

                        let arguments = message.arguments.to_vec();

                        let _result = tokio::spawn(async move {
                            #[cfg(feature = "with-telemetry")]
                            let mut signal_r_telemetry = crate::SignalRTelemetry::new(ctx_spawned);
                            signal_r_callbacks
                                .on(
                                    connection_spawned,
                                    message.headers,
                                    target,
                                    arguments,
                                    #[cfg(feature = "with-telemetry")]
                                    &mut signal_r_telemetry,
                                )
                                .await;

                            #[cfg(feature = "with-telemetry")]
                            (signal_r_telemetry)
                        })
                        .await;

                        #[cfg(feature = "with-telemetry")]
                        match _result {
                            Ok(my_telemetry) => {
                                if my_telemetry.get_write_telemetry() {
                                    my_telemetry::TELEMETRY_INTERFACE
                                        .write_success(
                                            &ctx,
                                            started,
                                            format!("[SignalR] {}", message.target),
                                            format!("Executed Ok",),
                                            my_telemetry
                                                .tags
                                                .add_ip(my_web_socket.addr.ip().to_string())
                                                .build(),
                                        )
                                        .await;
                                }
                            }
                            Err(err) => {
                                my_telemetry::TELEMETRY_INTERFACE
                                    .write_fail(
                                        &ctx,
                                        started,
                                        message.target.to_string(),
                                        format!("{:?}", err),
                                        TelemetryEventTagsBuilder::new()
                                            .add_ip(my_web_socket.addr.ip().to_string())
                                            .build(),
                                    )
                                    .await;
                            }
                        }
                    }

                    if packet_type == "6" {
                        signal_r_connection.send_ping_payload().await;
                    }
                } else {
                    read_first_payload(signal_r_connection, value).await
                }
            }
        }
    }
}

fn get_payload_type<'s>(first_line_reader: &'s mut JsonFirstLineReader<SliceIterator>) -> &'s str {
    while let Some(line) = first_line_reader.get_next() {
        let line = line.unwrap();
        if line.name.as_unescaped_name(first_line_reader).unwrap() == "type" {
            let result = line.value.as_unescaped_str(first_line_reader).unwrap();
            return result;
        }
    }

    panic!("Packet type is not found")
}

async fn read_first_payload<TCtx: Send + Sync + Default + 'static>(
    signal_r_connection: &Arc<MySignalRConnection<TCtx>>,
    payload: &str,
) {
    let mut json_reader: JsonFirstLineReader<SliceIterator> = payload.as_bytes().into();

    let mut protocol = false;
    let mut version = false;
    while let Some(line) = json_reader.get_next() {
        let line = line.unwrap();

        if line.name.as_unescaped_name(&json_reader).unwrap() == "protocol" {
            protocol = true;
        }
        if line.name.as_unescaped_name(&json_reader).unwrap() == "version" {
            version = true;
        }
    }

    if protocol == true && version == true {
        signal_r_connection.set_has_greeting();
        signal_r_connection.send_raw_payload("{}".to_string()).await;
    }
}
