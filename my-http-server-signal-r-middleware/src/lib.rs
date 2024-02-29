pub mod messages;
mod middleware;
mod middleware_builder;
mod my_signal_r_action_callback;
mod my_signal_r_actions;
mod my_signal_r_callbacks;
mod process_connect;
mod process_disconnect;
mod signal_r_connection;
mod signal_r_connections_list;
mod signal_r_liveness_loop;
mod signal_r_message_publisher;
mod signal_r_publishers_builder;
mod signal_r_serializer;
mod tags;
mod web_socket_callbacks;
pub use middleware::*;
pub use middleware_builder::*;
pub use my_signal_r_action_callback::*;
pub use my_signal_r_callbacks::*;
use process_connect::process_connect;
use process_disconnect::process_disconnect;
pub use signal_r_connection::*;
pub use signal_r_connections_list::*;
pub use signal_r_message_publisher::*;
pub use signal_r_publishers_builder::*;
pub use signal_r_serializer::*;
pub use tags::Tags;

pub use web_socket_callbacks::WebSocketCallbacks;
mod signal_r_param;
pub use signal_r_param::*;
#[cfg(feature = "with-telemetry")]
mod signal_r_telemetry;
#[cfg(feature = "with-telemetry")]
pub use signal_r_telemetry::*;

pub struct SignalRConnectionId(String);

impl SignalRConnectionId {
    pub fn generate() -> Self {
        let mut connection_id = uuid::Uuid::new_v4().to_string();
        connection_id = connection_id.replace("-", "");

        Self(connection_id)
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_ref_of_string(&self) -> &String {
        &self.0
    }
}

pub struct SignalRConnectionToken(String);
impl SignalRConnectionToken {
    pub fn generate() -> Self {
        let mut connection_token = uuid::Uuid::new_v4().to_string();
        connection_token = connection_token.replace("-", "");

        Self(connection_token)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_ref_of_string(&self) -> &String {
        &self.0
    }
}
