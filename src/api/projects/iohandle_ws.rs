use anyhow::anyhow;
use axum::{
    extract::{
        ws::{close_code, CloseFrame, Message as WsMessage, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{error::RecvError, Receiver};

use std::{borrow::Cow, net::SocketAddr};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

//allows to split the websocket stream into separate TX and RX branches

use tracing::{error, info, trace, warn};

use crate::{api::error::ApiError, SharedAppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscribeKind {
    #[serde(rename = "stdout")]
    StdOut,
    #[serde(rename = "stderr")]
    StdErr,
}

pub async fn ws_upgrader(
    State(state): State<SharedAppState>,
    Path((id, kind)): Path<(usize, SubscribeKind)>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, ApiError> {
    let handle = state
        .io_executor
        .get_handle_by_id(id)
        .await
        .ok_or(ApiError::new(
            StatusCode::NOT_FOUND,
            anyhow::Error::msg("could not find handle w/ that id"),
        ))?;

    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    trace!(address = %addr, "upgrading connection");
    match kind {
        SubscribeKind::StdOut => Ok::<hyper::Response<axum::body::Body>, ()>(
            ws.on_upgrade(move |socket| handle_socket(socket, handle.stdout, addr)),
        ),
        SubscribeKind::StdErr => Ok::<hyper::Response<axum::body::Body>, ()>(
            ws.on_upgrade(move |socket| handle_socket(socket, handle.stderr, addr)),
        ),
    }
    .map_err(|_| ApiError::new(StatusCode::BAD_REQUEST, anyhow!("ws upgrade error")))
}

pub async fn handle_socket(
    mut socket: WebSocket,
    mut stdstream: Receiver<String>,
    adress: SocketAddr,
) {
    loop {
        match stdstream.recv().await {
            Ok(l) => {
                if let Err(error) = socket.send(WsMessage::Text(l)).await {
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
            Err(RecvError::Lagged(c)) => warn!(%adress, missed = c, "ws stream lagged"),
            Err(_) => {
                info!(%adress, "io handle closed");
                socket
                    .send(WsMessage::Close(Some(CloseFrame {
                        code: close_code::AWAY,
                        reason: Cow::from("io closed"),
                    })))
                    .await
                    .unwrap();
                break;
            }
        }
    }
}
