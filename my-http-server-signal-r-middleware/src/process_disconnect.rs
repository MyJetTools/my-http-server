use std::sync::Arc;

use crate::{MySignalRCallbacks, MySignalRConnection, SignalRConnectionsList};

pub async fn process_disconnect<TCtx: Send + Sync + Default + 'static>(
    sockets_list: &Arc<SignalRConnectionsList<TCtx>>,
    signal_r_connection: &Arc<MySignalRConnection<TCtx>>,
    connect_events: Arc<dyn MySignalRCallbacks<TCtx = TCtx> + Send + Sync + 'static>,
) {
    let removed_connection = sockets_list
        .remove(signal_r_connection.get_list_index())
        .await;

    if let Some(removed_connection) = removed_connection {
        #[cfg(feature = "debug_ws")]
        println!(
            "SignalR {} is disconnected with connection token {:?}",
            removed_connection.connection_id, removed_connection.connection_token
        );
        connect_events.disconnected(&removed_connection).await;
    }
}
