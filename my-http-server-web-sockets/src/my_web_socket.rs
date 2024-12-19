use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
};

use futures::SinkExt;
use futures_util::stream::SplitSink;
use hyper_tungstenite::{
    tungstenite::{Error, Message},
    HyperWebsocketStream,
};
use my_http_server_core::UrlEncodedData;
use rust_extensions::{date_time::DateTimeAsMicroseconds, Logger};
use tokio::sync::Mutex;

use crate::MyWebSocketCallback;

pub struct MyWebSocket {
    pub write_stream: Mutex<Option<SplitSink<HyperWebsocketStream, Message>>>,
    pub addr: SocketAddr,
    pub id: i64,
    callbacks: Arc<dyn MyWebSocketCallback + Send + Sync + 'static>,
    query_string: Option<String>,
    connected: AtomicBool,
    logs: Arc<dyn Logger + Send + Sync + 'static>,
    connected_at: DateTimeAsMicroseconds,
}

impl MyWebSocket {
    pub fn new(
        id: i64,
        addr: SocketAddr,
        write_stream: SplitSink<HyperWebsocketStream, Message>,
        query_string: Option<String>,
        callbacks: Arc<dyn MyWebSocketCallback + Send + Sync + 'static>,
        logs: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        Self {
            write_stream: Mutex::new(write_stream.into()),
            addr,
            id,
            query_string,
            connected: AtomicBool::new(true),
            callbacks,
            logs,
            connected_at: DateTimeAsMicroseconds::now(),
        }
    }

    async fn send_messages_if_connected(
        &self,
        msgs: impl Iterator<Item = Message>,
    ) -> Result<(), Error> {
        let mut write_access = self.write_stream.lock().await;
        if let Some(stream) = &mut *write_access {
            for msg in msgs {
                let result = stream.send(msg).await;

                if result.is_err() {
                    return result;
                }
            }
        }

        Ok(())
    }

    pub async fn send_message(&self, msg: impl Iterator<Item = Message>) {
        let result = self.send_messages_if_connected(msg).await;

        if let Err(err) = result {
            let mut ctx = HashMap::new();

            ctx.insert("Ip".to_string(), self.addr.to_string());
            ctx.insert("Id".to_string(), self.id.to_string());
            ctx.insert("ConnectedAt".to_string(), self.connected_at.to_rfc3339());
            self.logs.write_warning(
                "Error sending message to websocket".to_string(),
                format!("{err}"),
                Some(ctx),
            );

            self.disconnect().await;
        }
    }

    pub fn get_query_string<'s>(&'s self) -> Option<UrlEncodedData<'s>> {
        let str = self.query_string.as_ref()?;

        match UrlEncodedData::from_query_string(str) {
            Ok(result) => Some(result),
            Err(_) => {
                let mut ctx = HashMap::new();

                ctx.insert("Ip".to_string(), self.addr.to_string());
                ctx.insert("Id".to_string(), self.id.to_string());
                ctx.insert("ConnectedAt".to_string(), self.connected_at.to_rfc3339());
                self.logs.write_warning(
                    "WebSocket parsing query string".to_string(),
                    format!("Invalid query string {}", str.to_string()),
                    Some(ctx),
                );
                return None;
            }
        }
    }

    pub async fn disconnect(&self) -> bool {
        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let just_disconnected = {
            let mut write_access = self.write_stream.lock().await;
            if let Some(mut item) = write_access.take() {
                let result = item.close().await;

                if let Err(err) = result {
                    println!("Can not close websocket {}. Reason: {:?}", self.id, err);
                }
                true
            } else {
                false
            }
        };

        if just_disconnected {
            self.callbacks.disconnected(self).await;
        }

        just_disconnected
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}
