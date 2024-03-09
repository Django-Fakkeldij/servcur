use axum::{
    extract::{
        ws::{close_code, CloseFrame, Message as WsMessage, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::IntoResponse,
};
use bollard::{
    container::{LogOutput, LogsOptions},
    errors::Error,
};
use futures_util::Stream;
use serde::Deserialize;

use std::{borrow::Cow, net::SocketAddr};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

//allows to split the websocket stream into separate TX and RX branches
use futures::stream::StreamExt;

use tracing::{error, trace, warn};

use crate::SharedAppState;

#[derive(Deserialize, Debug)]
pub struct LogPage {
    pub since: Option<usize>,
}

pub async fn ws_upgrader(
    State(state): State<SharedAppState>,
    Query(query): Query<LogPage>,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let stream;
    {
        let since = query.since.unwrap();
        warn!(%since, "Since");
        stream = state.lock_owned().await.state.logs(
            &id,
            Some(LogsOptions::<String> {
                stdout: true,
                stderr: true,
                timestamps: true,
                since: since as i64,
                follow: true,
                ..Default::default()
            }),
        );
    }
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    trace!(address = %addr, "upgrading connection");
    Ok::<hyper::Response<axum::body::Body>, ()>(
        ws.on_upgrade(move |socket| handle_socket(stream, socket, addr)),
    )
}

pub async fn handle_socket(
    mut stream: impl Stream<Item = Result<LogOutput, Error>> + Unpin,
    mut socket: WebSocket,
    adress: SocketAddr,
) {
    loop {
        match stream.next().await {
            Some(Ok(log)) => {
                if let Err(error) = socket.send(WsMessage::Text(log.to_string())).await {
                    error!(%adress, %error, "ws sending err to send; closing ws");
                    socket
                        .send(WsMessage::Close(Some(CloseFrame {
                            code: close_code::AWAY,
                            reason: Cow::from("ws sending error"),
                        })))
                        .await
                        .unwrap();
                    break;
                }
            }
            Some(Err(error)) => {
                error!(%adress, %error, "log reading err to send; closing ws");
                socket
                    .send(WsMessage::Close(Some(CloseFrame {
                        code: close_code::AWAY,
                        reason: Cow::from("log error"),
                    })))
                    .await
                    .unwrap();
                break;
            }
            None => {}
        }
    }
}
