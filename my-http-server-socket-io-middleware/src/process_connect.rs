use std::sync::Arc;

use my_http_server_web_sockets::MyWebSocket;
use socket_io_utils::{SocketIoHandshakeOpenModel, SocketIoSettings};

use crate::{socket_io_list::SocketIoList, MySocketIoCallbacks, MySocketIoConnection};

pub async fn process_connect(
    connections_callback: &Arc<dyn MySocketIoCallbacks + Send + Sync + 'static>,
    socket_io_list: &Arc<SocketIoList>,
    settings: &SocketIoSettings,
    web_socket: Option<Arc<MyWebSocket>>,
) -> (Arc<MySocketIoConnection>, SocketIoHandshakeOpenModel) {
    let sid = uuid::Uuid::new_v4().to_string();

    let sid = sid.replace("-", "")[..8].to_string();

    let handshake_model = SocketIoHandshakeOpenModel::from_settings(sid.clone(), settings);

    let socket_io = MySocketIoConnection::new(sid, web_socket);
    let socket_io_connection = Arc::new(socket_io);

    connections_callback
        .connected(socket_io_connection.clone())
        .await
        .unwrap();

    socket_io_list
        .add_socket_io(socket_io_connection.clone())
        .await;

    (socket_io_connection, handshake_model)
}
