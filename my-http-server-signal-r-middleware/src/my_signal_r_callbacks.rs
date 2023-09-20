use std::{collections::HashMap, sync::Arc};

use my_http_server_core::HttpFailResult;

use crate::MySignalRConnection;

#[async_trait::async_trait]
pub trait MySignalRCallbacks {
    type TCtx: Send + Sync + Default + 'static;
    async fn connected(
        &self,
        connection: &Arc<MySignalRConnection<Self::TCtx>>,
    ) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, connection: &Arc<MySignalRConnection<Self::TCtx>>);
    async fn on_ping(&self, connection: &Arc<MySignalRConnection<Self::TCtx>>);
    async fn on(
        &self,
        connection: Arc<MySignalRConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: String,
        data: Vec<u8>,
        #[cfg(feature = "with-telemetry")] ctx: &mut crate::SignalRTelemetry,
    );
}

#[async_trait::async_trait]
pub trait MySignalRTransportCallbacks {
    type TCtx: Send + Sync + Default + 'static;
    async fn connected(
        &self,
        connection: &Arc<MySignalRConnection<Self::TCtx>>,
    ) -> Result<(), HttpFailResult>;
    async fn disconnected(&self, connection: &Arc<MySignalRConnection<Self::TCtx>>);
    async fn on_ping(&self, connection: &Arc<MySignalRConnection<Self::TCtx>>);
}

#[async_trait::async_trait]
pub trait MySignalRPayloadCallbacks {
    type TCtx: Send + Sync + Default + 'static;
    async fn on(
        &self,
        signal_r_connection: &Arc<MySignalRConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: &str,
        data: &[u8],
        #[cfg(feature = "with-telemetry")] ctx: &mut crate::SignalRTelemetry,
    );
}
