use std::{collections::HashMap, sync::Arc};

use my_json::json_reader::array_iterator::JsonArrayIterator;
use rust_extensions::{array_of_bytes_iterator::SliceIterator, Logger};

use crate::{MySignalRConnection, MySignalRPayloadCallbacks, SignalRContractSerializer};

#[async_trait::async_trait]
pub trait MySignalRActionSubscriber<
    TContract: SignalRContractSerializer<Item = TContract> + Send + Sync + 'static,
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
    TContract: SignalRContractSerializer + Send + Sync + 'static,
    TCtx: Send + Sync + Default + 'static,
> {
    pub callback:
        Arc<dyn MySignalRActionSubscriber<TContract, TCtx = TCtx> + Send + Sync + 'static>,
    pub logger: Arc<dyn Logger + Send + Sync + 'static>,
}

#[async_trait::async_trait]
impl<
        TContract: SignalRContractSerializer<Item = TContract> + Send + Sync + 'static,
        TCtx: Send + Sync + Default + 'static,
    > MySignalRPayloadCallbacks for MySignalRCallbacksInstance<TContract, TCtx>
{
    type TCtx = TCtx;

    async fn on(
        &self,
        connection: &Arc<MySignalRConnection<Self::TCtx>>,
        headers: Option<HashMap<String, String>>,
        data: &[u8],
        #[cfg(feature = "with-telemetry")] ctx: &mut crate::SignalRTelemetry,
    ) {
        let mut params = Vec::new();

        let mut json_array_iterator: JsonArrayIterator<SliceIterator> = data.into();

        while let Some(line) = json_array_iterator.get_next() {
            match line {
                Ok(itm) => params.push(itm),
                Err(err) => {
                    let mut ctx = HashMap::new();
                    ctx.insert("action".to_string(), TContract::ACTION_NAME.to_string());
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

        match TContract::deserialize(params.iter().map(|x| x.as_bytes(&json_array_iterator))) {
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
                ctx.insert("action".to_string(), TContract::ACTION_NAME.to_string());
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
