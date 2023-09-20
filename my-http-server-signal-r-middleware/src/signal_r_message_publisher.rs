use std::sync::Arc;

use crate::{MySignalRConnection, SignalRConnectionsList, SignalRParam};

pub trait SignalRContractSerializer {
    fn serialize(self) -> Vec<Vec<u8>>;
}
pub struct SignalRMessagePublisher<
    TContract: SignalRContractSerializer + Send + Sync + 'static,
    TCtx: Default + Send + Sync + 'static,
> {
    signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
    itm: std::marker::PhantomData<TContract>,
    action_name: String,
}

impl<
        TContract: SignalRContractSerializer + Send + Sync + 'static,
        TCtx: Default + Send + Sync + 'static,
    > SignalRMessagePublisher<TContract, TCtx>
{
    pub fn new(action_name: String, signal_r_list: Arc<SignalRConnectionsList<TCtx>>) -> Self {
        Self {
            action_name,
            signal_r_list,
            itm: std::marker::PhantomData,
        }
    }

    pub async fn broadcast_to_all(&self, contract: TContract) {
        if let Some(connections) = self.signal_r_list.get_all().await {
            let payload = contract.serialize();

            for connection in connections {
                let params = SignalRParam::Raw(payload.as_slice());

                connection.send(self.action_name.as_str(), &params).await;
            }
        }
    }

    pub async fn send_to_connection(
        &self,
        connection: &MySignalRConnection<TCtx>,
        contract: TContract,
    ) {
        let payload = contract.serialize();
        let params = SignalRParam::Raw(payload.as_slice());
        connection.send(self.action_name.as_str(), &params).await;
    }

    pub async fn send_to_tagged_connections(&self, key: &str, contract: TContract) {
        if let Some(connections) = self.signal_r_list.get_tagged_connections(key).await {
            let payload = contract.serialize();

            for connection in connections {
                let params = SignalRParam::Raw(payload.as_slice());
                connection.send(self.action_name.as_str(), &params).await;
            }
        }
    }

    pub async fn send_to_tagged_connections_with_value(
        &self,
        key: &str,
        value: &str,
        contract: TContract,
    ) {
        if let Some(connections) = self
            .signal_r_list
            .get_tagged_connections_with_value(key, value)
            .await
        {
            let payload = contract.serialize();

            for connection in connections {
                let params = SignalRParam::Raw(payload.as_slice());
                connection.send(self.action_name.as_str(), &params).await;
            }
        }
    }
}
