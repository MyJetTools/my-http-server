use std::{sync::Arc, time::Duration};

use hyper_tungstenite::tungstenite::Message;
use my_http_server_core::*;
use my_http_server_web_sockets::MyWebSocket;

use socket_io_utils::{SocketIoContract, SocketIoMessage, SocketIoSettings};

use crate::{
    namespaces::SocketIoNameSpaces, socket_io_list::SocketIoList, MySocketIoCallbacks,
    MySocketIoConnection,
};

pub struct WebSocketCallbacks {
    pub socket_io_list: Arc<SocketIoList>,
    pub registered_sockets: Arc<SocketIoNameSpaces>,
    pub connections_callback: Arc<dyn MySocketIoCallbacks + Send + Sync + 'static>,
    pub settings: Arc<SocketIoSettings>,
}

impl WebSocketCallbacks {
    async fn handle_socket_io_message(
        &self,
        socket_io: Arc<MySocketIoConnection>,
        msg: SocketIoMessage,
    ) {
        match msg {
            SocketIoMessage::Connect { namespace, sid } => {
                println!(
                    "Namespace: {}, SID: {:?}",
                    namespace.as_str(),
                    sid.map(|itm| itm.to_string())
                );
            }
            SocketIoMessage::Disconnect { namespace } => {
                println!("Disconnect: {}.", namespace.as_str(),);
            }
            SocketIoMessage::Event {
                namespace,
                event_name,
                data,
                ack,
            } => {
                let response = self
                    .connections_callback
                    .on_callback(
                        &socket_io,
                        namespace.as_str(),
                        event_name.as_str(),
                        data.as_str(),
                    )
                    .await;

                if let Some(ack) = ack {
                    let response = SocketIoMessage::Ack {
                        namespace,
                        event_name,
                        data: response.unwrap_or_default().into(),
                        ack,
                    };

                    socket_io.send_message(&response.into()).await;
                }
            }
            SocketIoMessage::Ack {
                namespace: _,
                data: _,
                event_name: _,
                ack: _,
            } => {}
            SocketIoMessage::ConnectError { namespace, message } => {
                println!(
                    "Namespace: {}, ConnectError: {}",
                    namespace.as_str(),
                    message.as_str()
                );
            }
        }

        /*
        let nsp_str = msg.get_namespace();

        if let Some(socket) = self.registered_sockets.get(nsp_str).await {
            let mut event_name = None;
            let mut event_data = None;

            let mut i = 0;

            let json_array = JsonArrayIterator::new(msg.data.as_bytes().into()).unwrap();

            while let Some(next) = json_array.get_next() {
                let data = next.unwrap();
                if i == 0 {
                    event_name = Some(data);
                } else if i == 1 {
                    event_data = Some(data);
                } else {
                    break;
                }

                i += 1;
            }

            let event_name = event_name.unwrap();
            let event_data = event_data.unwrap();

            if let Some(ack_data) = socket
                .on(
                    event_name.as_str().unwrap().as_str(),
                    event_data.as_str().unwrap().as_str(),
                )
                .await
            {
                let ack_contract = MySocketIoMessage::Ack(MySocketIoTextPayload {
                    nsp: msg.nsp,
                    data: ack_data,
                    id: msg.id,
                });

                socket_io.send_message(&ack_contract).await;
            }

        }
        */
    }
}

#[async_trait::async_trait]
impl my_http_server_web_sockets::MyWebSocketCallback for WebSocketCallbacks {
    async fn connected(
        &self,
        my_web_socket: Arc<MyWebSocket>,
        _disconnect_timeout: Duration,
    ) -> Result<(), HttpFailResult> {
        #[cfg(feature = "debug-ws")]
        println!("connected web_socket:{}", my_web_socket.id);

        if let Some(query_string) = my_web_socket.get_query_string() {
            let sid = query_string.get_optional("sid");

            if sid.is_none() {
                let (socket_io, response) = crate::process_connect(
                    &self.connections_callback,
                    &self.socket_io_list,
                    &self.settings,
                    Some(my_web_socket.clone()),
                )
                .await;

                let payload = SocketIoContract::Open(response).serialize();

                my_web_socket
                    .send_message([Message::Text(payload.text_frame.into())].into_iter())
                    .await;

                let settings = self.settings.clone();

                tokio::spawn(super::socket_io_livness_loop::start(
                    self.connections_callback.clone(),
                    self.socket_io_list.clone(),
                    socket_io,
                    settings,
                ));
                return Ok(());
            }

            let sid = sid.unwrap();

            let sid = sid.as_str()?;

            match self
                .socket_io_list
                .assign_web_socket_to_socket_io(sid.as_str(), my_web_socket.clone())
                .await
            {
                Some(socket_io) => {
                    let settings = self.settings.clone();
                    tokio::spawn(super::socket_io_livness_loop::start(
                        self.connections_callback.clone(),
                        self.socket_io_list.clone(),
                        socket_io,
                        settings,
                    ));
                }
                None => {
                    my_web_socket
                        .send_message(
                            [Message::Text(
                                format!("Socket.IO with id {} is not found", sid.as_str(),).into(),
                            )]
                            .into_iter(),
                        )
                        .await;

                    return Ok(());
                }
            };
        }

        Ok(())
    }

    async fn disconnected(&self, my_web_socket: &MyWebSocket) {
        #[cfg(feature = "debug-ws")]
        println!("disconnected web_socket:{}", my_web_socket.id);
        let find_result = self
            .socket_io_list
            .get_by_web_socket_id(my_web_socket.id)
            .await;

        if let Some(socket_io) = find_result {
            crate::process_disconnect(&self.socket_io_list, &socket_io, &self.connections_callback)
                .await;
        }
    }
    async fn on_message(&self, my_web_socket: Arc<MyWebSocket>, message: Message) {
        #[cfg(feature = "debug-ws")]
        println!("Websocket{}, MSG: {:?}", my_web_socket.id, message);

        let socket_io_connection = self
            .socket_io_list
            .get_by_web_socket_id(my_web_socket.id)
            .await
            .unwrap();

        socket_io_connection.update_incoming_activity();
        //if let Some(socket_io_ref) = socket_io.as_ref() {}

        if let Message::Text(value) = &message {
            let contract = socket_io_utils::SocketIoContract::deserialize(value.as_str());

            match contract {
                socket_io_utils::SocketIoContract::Open(_) => {
                    println!("Open is a server side message")
                }
                socket_io_utils::SocketIoContract::Close => {
                    println!("Close is a server side message")
                }
                socket_io_utils::SocketIoContract::Ping { with_probe: _ } => {}
                socket_io_utils::SocketIoContract::Pong { with_probe: _ } => {}
                socket_io_utils::SocketIoContract::Message(message) => {
                    self.handle_socket_io_message(socket_io_connection, message)
                        .await;
                }
                socket_io_utils::SocketIoContract::Upgrade => {
                    println!("Upgrade is a server side message")
                }
                socket_io_utils::SocketIoContract::Noop => {}
            }
        }
    }
}
