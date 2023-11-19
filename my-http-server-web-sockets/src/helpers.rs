use std::{sync::Arc, time::Duration};

use futures::stream::SplitStream;
use futures::StreamExt;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use my_http_server_core::HttpRequest;
use my_http_server_core::{HttpFailResult, HttpOkResult, HttpOutput, WebContentType};

use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::{my_web_socket_callback::WebSocketMessage, MyWebSocket, MyWebSocketCallback};

pub async fn handle_web_socket_upgrade<
    TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static,
>(
    req: &mut HttpRequest,
    callback: Arc<TMyWebSocketCallback>,
    id: i64,
    disconnect_timeout: Duration,
) -> Result<HttpOkResult, HttpFailResult> {
    let query_string = if let Some(query_string) = req.get_uri().query() {
        Some(query_string.to_string())
    } else {
        None
    };

    let addr = req.addr.clone();

    let req = req.take_incoming_body();

    let upgrade_result = crate::web_sockets_upgrade::upgrade(req).await;

    if let Err(err) = upgrade_result {
        let content = format!("Can not upgrade websocket. Reason: {}", err);
        println!("{}", content);
        return Err(HttpFailResult::new(
            WebContentType::Text,
            400,
            content.into_bytes(),
            false,
            false,
        ));
    }

    let (upgraded, response) = upgrade_result.unwrap();

    if let Some(upgraded) = upgraded {
        tokio::spawn(async move {
            let upgraded = TokioIo::new(upgraded);

            let ws_stream = tokio_tungstenite::accept_async(upgraded).await.unwrap();

            let (ws_sender, ws_receiver) = ws_stream.split();

            let my_web_socket = MyWebSocket::new(id, addr, ws_sender, query_string);

            let my_web_socket = Arc::new(my_web_socket);

            callback
                .connected(my_web_socket.clone(), disconnect_timeout)
                .await
                .unwrap();

            let serve_socket_result = tokio::spawn(serve_websocket(
                my_web_socket.clone(),
                ws_receiver,
                callback.clone(),
            ))
            .await;

            callback.disconnected(my_web_socket.clone()).await;

            if let Err(err) = serve_socket_result {
                println!(
                    "Execution of websocket {} is finished with panic. {}",
                    id, err
                );
            }
        });
    }

    HttpOutput::Raw(response).into_ok_result(false)
}

/// Handle a websocket connection.
async fn serve_websocket<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>(
    my_web_socket: Arc<MyWebSocket>,
    mut read_stream: SplitStream<WebSocketStream<TokioIo<Upgraded>>>,
    callback: Arc<TMyWebSocketCallback>,
) -> Result<(), tungstenite::Error> {
    while let Some(message) = read_stream.next().await {
        let result = match message? {
            Message::Text(msg) => {
                send_message(
                    my_web_socket.clone(),
                    WebSocketMessage::String(msg),
                    callback.clone(),
                )
                .await
            }
            Message::Binary(msg) => {
                send_message(
                    my_web_socket.clone(),
                    WebSocketMessage::Binary(msg),
                    callback.clone(),
                )
                .await
            }
            Message::Ping(_) => Ok(()),
            Message::Pong(_) => Ok(()),
            Message::Close(_) => Ok(()),
            Message::Frame(_) => Ok(()),
        };

        if let Err(err) = result {
            eprintln!("Error in websocket connection: {}", err);
            break;
        }
    }

    Ok(())
}

async fn send_message<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>(
    web_socket: Arc<MyWebSocket>,
    message: WebSocketMessage,
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
