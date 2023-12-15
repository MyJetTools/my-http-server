use std::sync::Arc;

use my_http_server_web_sockets::MyWebSocket;

use crate::{
    MySignalRCallbacks, MySignalRConnection, SignalRConnectionId, SignalRConnectionToken,
    SignalRConnectionsList,
};

pub async fn process_connect<
    TCtx: Send + Sync + Default + 'static,
    TMySignalRCallbacks: MySignalRCallbacks<TCtx = TCtx> + Send + Sync + 'static,
>(
    connections_callback: &Arc<TMySignalRCallbacks>,
    signal_r_list: &Arc<SignalRConnectionsList<TCtx>>,
    negotiation_version: usize,
    web_socket: Option<Arc<MyWebSocket>>,
) -> (Arc<MySignalRConnection<TCtx>>, String) {
    let connection_id = SignalRConnectionId::generate();

    let connection_token = if negotiation_version == 0 {
        None
    } else {
        Some(SignalRConnectionToken::generate())
    };

    let result = crate::messages::generate_negotiate_response(
        negotiation_version,
        &connection_id,
        &connection_token,
    );

    let signal_r_connection = MySignalRConnection::new(
        connection_id,
        connection_token,
        negotiation_version,
        web_socket,
    );
    let signal_r_connection = Arc::new(signal_r_connection);

    connections_callback
        .connected(&signal_r_connection)
        .await
        .unwrap();

    signal_r_list
        .add_signal_r_connection(signal_r_connection.clone())
        .await;

    (signal_r_connection, result)
}
