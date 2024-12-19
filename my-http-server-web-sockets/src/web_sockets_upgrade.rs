use std::sync::Arc;
use std::{net::SocketAddr, time::Duration};

use futures::StreamExt;
use futures_util::stream::SplitStream;
use hyper::body::Bytes;

use http_body_util::Full;
use hyper::{Request, Response};
use hyper_tungstenite::tungstenite::Message;
use hyper_tungstenite::HyperWebsocketStream;
use rust_extensions::Logger;

use crate::{MyWebSocket, MyWebSocketCallback};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub async fn upgrade<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>(
    id: i64,
    addr: SocketAddr,
    query_string: Option<String>,
    req: Request<hyper::body::Incoming>,
    callback: Arc<TMyWebSocketCallback>,
    disconnect_timeout: Duration,
    logs: Arc<dyn Logger + Send + Sync + 'static>,
) -> Result<Response<Full<Bytes>>, Error> {
    let (response, websocket) = hyper_tungstenite::upgrade(req, None)?;

    tokio::spawn(async move {
        let ws_stream = websocket.await;

        match ws_stream {
            Ok(ws_stream) => {
                let (ws_sender, ws_receiver) = ws_stream.split();

                let my_web_socket =
                    MyWebSocket::new(id, addr, ws_sender, query_string, callback.clone(), logs);

                let my_web_socket = Arc::new(my_web_socket);

                callback
                    .connected(my_web_socket.clone(), disconnect_timeout)
                    .await
                    .unwrap();

                let my_web_socket_cloned = my_web_socket.clone();

                if let Err(e) = serve_websocket(my_web_socket_cloned, ws_receiver, callback).await {
                    eprintln!("Error in websocket connection: {e}");
                }
            }
            Err(err) => {
                println!("Error in websocket connection: {}", err);
            }
        }
    });

    Ok(response)
}

/// Handle a websocket connection.
async fn serve_websocket<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>(
    my_web_socket: Arc<MyWebSocket>,
    mut websocket: SplitStream<HyperWebsocketStream>,
    callback: Arc<TMyWebSocketCallback>,
) -> Result<(), Error> {
    while let Some(message) = websocket.next().await {
        callback_message(my_web_socket.clone(), message?, callback.clone()).await?;
    }

    my_web_socket.disconnect().await;

    Ok(())
}

async fn callback_message<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>(
    web_socket: Arc<MyWebSocket>,
    message: Message,
    callback: Arc<TMyWebSocketCallback>,
) -> Result<(), String> {
    let result = tokio::spawn(async move {
        callback.on_message(web_socket, message).await;
    })
    .await;

    if let Err(err) = result {
        return Err(format!("Error in on_message: {}", err));
    }

    Ok(())
}
