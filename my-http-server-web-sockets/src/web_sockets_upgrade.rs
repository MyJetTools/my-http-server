use std::pin::Pin;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use futures::FutureExt;
use futures::StreamExt;
use std::panic::AssertUnwindSafe;
use futures_util::stream::SplitStream;

use hyper_tungstenite::tungstenite::protocol::CloseFrame;
use hyper_tungstenite::tungstenite::protocol::Role;
use hyper_tungstenite::tungstenite::Message;
use hyper_tungstenite::HyperWebsocketStream;
use hyper_tungstenite::WebSocketStream;
use rust_extensions::Logger;

use crate::{MyWebSocket, MyWebSocketCallback, MyWebSocketHttpRequest, WsMessage};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

type WsStreamFuture =
    Pin<Box<dyn std::future::Future<Output = Result<HyperWebsocketStream, Error>> + Send>>;

use my_http_server_core::{my_hyper_utils::*, MyHyperHttpRequest, SocketAddress};

fn is_h2_ws_connect_request(req: &MyHyperHttpRequest) -> bool {
    let method = match req {
        MyHyperHttpRequest::Incoming(r) => r.method(),
        MyHyperHttpRequest::Full(r) => r.method(),
    };
    if method != hyper::Method::CONNECT {
        return false;
    }
    match req.extensions().get::<hyper::ext::Protocol>() {
        Some(protocol) => protocol.as_str().eq_ignore_ascii_case("websocket"),
        None => false,
    }
}

fn h2_ext_connect_upgrade(req: MyHyperHttpRequest) -> Result<(MyHttpResponse, WsStreamFuture), Error>
{
    let on_upgrade = match req {
        MyHyperHttpRequest::Incoming(mut req) => hyper::upgrade::on(&mut req),
        MyHyperHttpRequest::Full(mut req) => hyper::upgrade::on(&mut req),
    };

    let response = (
        hyper::Response::builder().status(hyper::StatusCode::OK),
        &[][..],
    )
        .to_my_http_response();

    let ws_future: WsStreamFuture = Box::pin(async move {
        let upgraded = on_upgrade.await?;
        let io = hyper_util::rt::TokioIo::new(upgraded);
        let stream = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
        Ok::<_, Error>(stream)
    });

    Ok((response, ws_future))
}

fn h1_websocket_upgrade(req: MyHyperHttpRequest) -> Result<(MyHttpResponse, WsStreamFuture), Error>
{
    let (response, websocket) = match req {
        MyHyperHttpRequest::Incoming(req) => hyper_tungstenite::upgrade(req, None)?,
        MyHyperHttpRequest::Full(req) => hyper_tungstenite::upgrade(req, None)?,
    };

    let ws_future: WsStreamFuture = Box::pin(async move {
        let stream = websocket.await?;
        Ok::<_, Error>(stream)
    });

    Ok((response.to_my_http_response(), ws_future))
}

pub async fn upgrade<TMyWebSocketCallback: MyWebSocketCallback + Send + Sync + 'static>(
    id: i64,
    addr: SocketAddress,
    query_string: Option<String>,
    req: MyHyperHttpRequest,
    callback: Arc<TMyWebSocketCallback>,
    disconnect_timeout: Duration,
    logs: Arc<dyn Logger + Send + Sync + 'static>,
) -> Result<MyHttpResponse, Error> {
    let http_request = MyWebSocketHttpRequest::new(&req, addr.clone());

    let (response, websocket) = if is_h2_ws_connect_request(&req) {
        h2_ext_connect_upgrade(req)?
    } else {
        h1_websocket_upgrade(req)?
    };

    tokio::spawn(async move {
        let ws_stream = websocket.await;

        let ws_stream = match ws_stream {
            Ok(ws_stream) => ws_stream,
            Err(_) => {
                let mut ctx = HashMap::new();
                ctx.insert("SocketId".to_string(), id.to_string());
                if let Some(query_string) = query_string {
                    ctx.insert("QueryString".to_string(), query_string);
                }

                /*
                logs.write_fatal_error(
                    "WebSocketUpgrade".to_string(),
                    format!("{:?}", err),
                    Some(ctx),
                );
                 */
                return;
            }
        };

        let (ws_sender, ws_receiver) = ws_stream.split();

        let my_web_socket = MyWebSocket::new(
            id,
            addr,
            ws_sender,
            query_string.clone(),
            callback.clone(),
            logs.clone(),
        );

        let my_web_socket = Arc::new(my_web_socket);

        let connected_result = callback
            .connected(my_web_socket.clone(), http_request, disconnect_timeout)
            .await;

        if let Err(err) = connected_result {
            if err.write_to_logs {
                let mut ctx = HashMap::new();
                ctx.insert("SocketId".to_string(), id.to_string());
                if let Some(query_string) = query_string {
                    ctx.insert("QueryString".to_string(), query_string);
                }

                /*
                logs.write_fatal_error(
                    "UpgradeWsSocket".to_string(),
                    format!("{:?}", err.reason),
                    Some(ctx),
                );
                 */
            }

            my_web_socket
                        .send_message(
                            [WsMessage::Close(Some(CloseFrame {
                                code: hyper_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Error,
                                reason: err.reason.into(),
                            }))]
                            .into_iter(),
                        )
                        .await;

            return;
        }

        let my_web_socket_cloned = my_web_socket.clone();

        if let Err(_) = serve_websocket(
            my_web_socket_cloned,
            ws_receiver,
            callback,
            disconnect_timeout,
        )
        .await
        {
            //println!("Error after serving websocket connection: {e}");
        }

        my_web_socket.disconnect().await;
    });

    Ok(response)
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
            break;
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
    let result = AssertUnwindSafe(callback.on_message(web_socket, message))
        .catch_unwind()
        .await;

    if let Err(err) = result {
        let msg = if let Some(s) = err.downcast_ref::<&'static str>() {
            (*s).to_string()
        } else if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else {
            "panic".to_string()
        };
        return Err(format!("Error in on_message: {}", msg));
    }

    Ok(())
}
