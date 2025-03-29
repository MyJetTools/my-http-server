use std::sync::Arc;
use std::{net::SocketAddr, time::Duration};

use futures::StreamExt;
use futures_util::stream::SplitStream;

use hyper::Request;
use hyper_tungstenite::tungstenite::Message;
use hyper_tungstenite::HyperWebsocketStream;
use rust_extensions::Logger;

use crate::{MyWebSocket, MyWebSocketCallback};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

use my_http_server_core::my_hyper_utils::*;

pub async fn upgrade<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>(
    id: i64,
    addr: SocketAddr,
    query_string: Option<String>,
    req: Request<hyper::body::Incoming>,
    callback: Arc<TMyWebSocketCallback>,
    disconnect_timeout: Duration,
    logs: Arc<dyn Logger + Send + Sync + 'static>,
) -> Result<MyHttpResponse, Error> {
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

                if let Err(e) = serve_websocket(
                    my_web_socket_cloned,
                    ws_receiver,
                    callback,
                    disconnect_timeout,
                )
                .await
                {
                    eprintln!("Error in websocket connection: {e}");
                }

                my_web_socket.disconnect().await;
            }
            Err(err) => {
                println!("Error in websocket connection: {}", err);
            }
        }
    });

    Ok(response.to_my_http_response())
}

/// Handle a websocket connection.
async fn serve_websocket<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>(
    my_web_socket: Arc<MyWebSocket>,
    mut websocket: SplitStream<HyperWebsocketStream>,
    callback: Arc<TMyWebSocketCallback>,
    disconnect_timeout: Duration,
) -> Result<(), Error> {
    loop {
        let future = websocket.next();

        let result = tokio::time::timeout(disconnect_timeout, future).await;

        if result.is_err() {
            let err = "No activity".to_string();
            return Err(err.into());
        }

        let message = result.unwrap();

        if message.is_none() {
            break;
        }

        let message = message.unwrap();

        let message = match message {
            Ok(message) => message,
            Err(err) => {
                println!("Getting WS message error:{}", err);
                my_web_socket.disconnect().await;
                return Err(err.into());
            }
        };

        let result = callback_message(my_web_socket.clone(), message, callback.clone()).await;

        if let Err(err) = result {
            my_web_socket.disconnect().await;
            return Err(err.into());
        }
    }

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
