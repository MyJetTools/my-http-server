use std::{collections::HashMap, sync::Arc};

use rust_extensions::Logger;

use crate::{MySignalRConnection, MySignalRPayloadCallbacks};

pub trait SignalRContractDeserializer {
    type Item;
    fn deserialize(data: &[&[u8]]) -> Result<Self::Item, String>;
}

#[async_trait::async_trait]
pub trait MySignalRActionCallbacks<
    TContract: SignalRContractDeserializer<Item = TContract> + Send + Sync + 'static,
>
{
    type TCtx: Send + Sync + Default + 'static;
    async fn on(
        &self,
        connection: &Arc<MySignalRConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        data: TContract,
        #[cfg(feature = "with-telemetry")] ctx: &mut crate::SignalRTelemetry,
    );
}

pub struct MySignalRCallbacksInstance<
    TContract: SignalRContractDeserializer<Item = TContract> + Send + Sync + 'static,
    TCtx: Send + Sync + Default + 'static,
> {
    pub action_name: String,
    pub callback: Arc<dyn MySignalRActionCallbacks<TContract, TCtx = TCtx> + Send + Sync + 'static>,
    pub logger: Arc<dyn Logger + Send + Sync + 'static>,
}

#[async_trait::async_trait]
impl<
        TContract: SignalRContractDeserializer<Item = TContract> + Send + Sync + 'static,
        TCtx: Send + Sync + Default + 'static,
    > MySignalRPayloadCallbacks for MySignalRCallbacksInstance<TContract, TCtx>
{
    type TCtx = TCtx;

    async fn on(
        &self,
        connection: &Arc<MySignalRConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        action_name: &str,
        data: &[u8],
        #[cfg(feature = "with-telemetry")] ctx: &mut crate::SignalRTelemetry,
    ) {
        let mut params = Vec::new();
        for item in my_json::json_reader::array_parser::JsonArrayIterator::new(data) {
            match item {
                Ok(itm) => params.push(itm),
                Err(err) => {
                    let mut ctx = HashMap::new();
                    ctx.insert("action".to_string(), action_name.to_string());
                    ctx.insert(
                        "payload".to_string(),
                        String::from_utf8_lossy(data).to_string(),
                    );
                    self.logger.write_fatal_error(
                        "SignalR payload handler".to_string(),
                        format!("Can read parameters payloads. Err: {:?}", err),
                        Some(ctx),
                    )
                }
            }
        }

        match TContract::deserialize(&params) {
            Ok(contract) => {
                self.callback
                    .on(
                        connection,
                        headers,
                        contract,
                        #[cfg(feature = "with-telemetry")]
                        ctx,
                    )
                    .await;
            }
            Err(err) => {
                let mut ctx = HashMap::new();
                ctx.insert("action".to_string(), action_name.to_string());
                ctx.insert(
                    "payload".to_string(),
                    String::from_utf8_lossy(data).to_string(),
                );
                self.logger.write_fatal_error(
                    "SignalR payload handler".to_string(),
                    format!("Can not deserialize payload. Err: {}", err),
                    Some(ctx),
                )
            }
        }
    }
}
