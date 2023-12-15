use std::{sync::Arc, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{MySignalRCallbacks, MySignalRConnection, SignalRConnectionsList};

pub async fn start<TCtx: Send + Sync + Default + 'static>(
    connect_events: Arc<dyn MySignalRCallbacks<TCtx = TCtx> + Send + Sync + 'static>,
    sockets_list: Arc<SignalRConnectionsList<TCtx>>,
    my_socket_io_connection: Arc<MySignalRConnection<TCtx>>,
    ping_disconnect: Duration,
) {
    #[cfg(feature = "debug_ws")]
    println!(
        "SignalR {} with connection token {:?} started liveness loop",
        my_socket_io_connection.connection_id, my_socket_io_connection.connection_token
    );

    while my_socket_io_connection.is_connected() {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let now = DateTimeAsMicroseconds::now();
        let last_incoming = my_socket_io_connection.get_last_incoming();

        if now.duration_since(last_incoming).as_positive_or_zero() > ping_disconnect {
            #[cfg(feature = "debug_ws")]
            println!(
                "SignalR {} with connection token {:?} looks like dead. Disconnecting",
                my_socket_io_connection.connection_id, my_socket_io_connection.connection_token
            );
            break;
        }
    }

    crate::process_disconnect(&sockets_list, &my_socket_io_connection, connect_events).await;
}
