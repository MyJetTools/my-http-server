use std::sync::Arc;

use crate::{MySignalRConnection, SignalRConnectionsList, SignalRContractSerializer, SignalRParam};

pub struct SignalRMessagePublisher<
    TContract: SignalRContractSerializer + Send + Sync + 'static,
    TCtx: Default + Send + Sync + 'static,
> {
    signal_r_list: Arc<SignalRConnectionsList<TCtx>>,
    itm: std::marker::PhantomData<TContract>,
}

impl<
        TContract: SignalRContractSerializer + Send + Sync + 'static,
        TCtx: Default + Send + Sync + 'static,
    > SignalRMessagePublisher<TContract, TCtx>
{
    pub fn new(signal_r_list: Arc<SignalRConnectionsList<TCtx>>) -> Self {
        Self {
            signal_r_list,
            itm: std::marker::PhantomData,
        }
    }

    pub async fn broadcast_to_all(&self, contract: TContract) {
        if let Some(connections) = self.signal_r_list.get_all().await {
            let payload = contract.serialize();

            for connection in connections {
                let params = SignalRParam::Raw(payload.as_slice());

                connection.send(TContract::ACTION_NAME, &params).await;
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
        connection.send(TContract::ACTION_NAME, &params).await;
    }

    pub async fn send_to_tagged_connections(&self, key: &str, contract: TContract) {
        if let Some(connections) = self.signal_r_list.get_tagged_connections(key).await {
            let payload = contract.serialize();

            for connection in connections {
                let params = SignalRParam::Raw(payload.as_slice());
                connection.send(TContract::ACTION_NAME, &params).await;
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
                connection.send(TContract::ACTION_NAME, &params).await;
            }
        }
    }
}
