use std::sync::Arc;

use rust_extensions::Logger;

use crate::{
    my_signal_r_actions::MySignalRActions, MySignalRActionSubscriber, MySignalRMiddleware,
    MySignalRTransportCallbacks, SignalRConnectionsList, SignalRContractSerializer,
};

pub struct MiddlewareBuilder<TCtx: Send + Sync + Default + 'static> {
    hub_name: String,
    signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
    actions: MySignalRActions<TCtx>,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
    disconnect_timeout: std::time::Duration,
}

impl<TCtx: Send + Sync + Default + 'static> MiddlewareBuilder<TCtx> {
    pub fn new(
        hub_name: String,
        signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        Self {
            hub_name,
            signal_r_list,
            actions: MySignalRActions::new(),
            logger,
            disconnect_timeout: std::time::Duration::from_secs(60),
        }
    }

    pub fn set_disconnect_timeout(mut self, disconnect_timeout: std::time::Duration) -> Self {
        self.disconnect_timeout = disconnect_timeout;
        self
    }

    pub fn with_transport_callback(
        mut self,
        transport_callback: Arc<
            dyn MySignalRTransportCallbacks<TCtx = TCtx> + Send + Sync + 'static,
        >,
    ) -> Self {
        if self.actions.transport_callbacks.is_some() {
            panic!("Transport callback is already registered");
        }

        self.actions.transport_callbacks = Some(transport_callback);
        self
    }

    pub fn with_action<
        TContract: SignalRContractSerializer<Item = TContract> + Send + Sync + 'static,
        TMySignalRPayloadCallbacks: MySignalRActionSubscriber<TContract, TCtx = TCtx> + Send + Sync + 'static,
    >(
        mut self,
        action: TMySignalRPayloadCallbacks,
    ) -> Self {
        self.actions.add_action(action, self.logger.clone());
        self
    }

    pub fn build(self) -> MySignalRMiddleware<TCtx> {
        MySignalRMiddleware::new(
            self.hub_name.as_str(),
            self.signal_r_list,
            self.actions,
            self.disconnect_timeout,
        )
    }
}
