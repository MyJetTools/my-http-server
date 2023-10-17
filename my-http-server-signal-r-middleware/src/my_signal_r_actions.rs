use std::{collections::HashMap, sync::Arc};

use my_http_server_core::HttpFailResult;
use rust_extensions::Logger;

use crate::{
    MySignalRActionSubscriber, MySignalRCallbacks, MySignalRCallbacksInstance, MySignalRConnection,
    MySignalRPayloadCallbacks, MySignalRTransportCallbacks, SignalRContractSerializer,
};

pub struct MySignalRActions<TCtx: Send + Sync + Default + 'static> {
    pub transport_callbacks:
        Option<Arc<dyn MySignalRTransportCallbacks<TCtx = TCtx> + Send + Sync + 'static>>,
    actions:
        HashMap<String, Arc<dyn MySignalRPayloadCallbacks<TCtx = TCtx> + Send + Sync + 'static>>,
}

impl<TCtx: Send + Sync + Default + 'static> MySignalRActions<TCtx> {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            transport_callbacks: None,
        }
    }

    pub fn add_action<
        TContract: SignalRContractSerializer<Item = TContract> + Send + Sync + 'static,
        TMySignalRPayloadCallbacks: MySignalRActionSubscriber<TContract, TCtx = TCtx> + Send + Sync + 'static,
    >(
        &mut self,
        callback: TMySignalRPayloadCallbacks,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        if self.actions.contains_key(TContract::ACTION_NAME) {
            panic!(
                "SignalR action already registered: {}",
                TContract::ACTION_NAME
            );
        }

        let instance = MySignalRCallbacksInstance {
            callback: Arc::new(callback),
            logger,
        };

        self.actions
            .insert(TContract::ACTION_NAME.to_string(), Arc::new(instance));
    }
}

#[async_trait::async_trait]
impl<TCtx: Send + Sync + Default + 'static> MySignalRCallbacks for MySignalRActions<TCtx> {
    type TCtx = TCtx;

    async fn connected(
        &self,
        connection: &Arc<MySignalRConnection<Self::TCtx>>,
    ) -> Result<(), HttpFailResult> {
        if let Some(c) = self.transport_callbacks.as_ref() {
            c.connected(connection).await
        } else {
            Ok(())
        }
    }
    async fn disconnected(&self, connection: &Arc<MySignalRConnection<Self::TCtx>>) {
        if let Some(c) = self.transport_callbacks.as_ref() {
            c.disconnected(connection).await
        }
    }

    async fn on_ping(&self, connection: &Arc<MySignalRConnection<Self::TCtx>>) {
        if let Some(c) = self.transport_callbacks.as_ref() {
            c.on_ping(connection).await
        }
    }
    async fn on(
        &self,
        signal_r_connection: Arc<MySignalRConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: String,
        data: Vec<u8>,
        #[cfg(feature = "with-telemetry")] ctx: &mut crate::SignalRTelemetry,
    ) {
        if let Some(action) = self.actions.get(action_name.as_str()) {
            action
                .on(
                    &signal_r_connection,
                    headers,
                    &data,
                    #[cfg(feature = "with-telemetry")]
                    ctx,
                )
                .await;
        } else {
            #[cfg(feature = "with-telemetry")]
            ctx.do_not_write_this_event()
        }
    }
}
