use std::sync::Arc;

use crate::{SignalRConnectionsList, SignalRContractSerializer, SignalRMessagePublisher};

pub struct SignalRPublishersBuilder<TCtx: Send + Sync + Default + 'static> {
    signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
}

impl<TCtx: Send + Sync + Default + 'static> SignalRPublishersBuilder<TCtx> {
    pub fn new(signal_r_list: Arc<SignalRConnectionsList<TCtx>>) -> Self {
        Self { signal_r_list }
    }
    pub fn get_publisher<TContract: SignalRContractSerializer + Send + Sync + 'static>(
        &self,
        action_name: String,
    ) -> SignalRMessagePublisher<TContract, TCtx> {
        return SignalRMessagePublisher::new(action_name, self.signal_r_list.clone());
    }
}
