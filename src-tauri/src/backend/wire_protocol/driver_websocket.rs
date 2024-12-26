//! Adapter for the Jupyter wire protocol over WebSocket.
//!
//! This protocol is documented in the `jupyter-server` project at
//! <https://jupyter-server.readthedocs.io/en/latest/developers/websocket-protocols.html>.
//!
//! It is very similar to the ZeroMQ protocol, but there is a thin framing layer
//! that allows messages to be sent over WebSocket binary payloads instead of
//! raw TCP sockets.

use std::sync::Arc;

use bytes::Bytes;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use reqwest::header::{HeaderValue, AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

use super::{KernelConnection, KernelHeader, KernelMessage};
use crate::Error;

// In this protocol, a kernel message is serialized over WebSocket as follows,
// where all integers are little-endian (indices refer to bytes):
//
// 0: offset_number (n+1)
// 8: offset_0
// 16: offset_1
// 8*offset_number: offset_n
// offset_0: channel
// offset_1: header
// offset_2: parent_header
// offset_3: metadata
// offset_4: content
// offset_5: buffer_0
// (offset_6: buffer_1 ... and so on)

fn to_ws_payload(msg: &KernelMessage, channel: &str) -> Option<Vec<u8>> {
    let offset_number = 5 + msg.buffers.len() as u64;
    let offset_0 = 8 * (offset_number + 1);
    let mut offsets = vec![offset_number];

    let mut payload = Vec::new();

    // offset_0: channel
    offsets.push(offset_0 + payload.len() as u64);
    payload.extend_from_slice(channel.as_bytes());

    // offset_1: header
    offsets.push(offset_0 + payload.len() as u64);
    payload.append(&mut serde_json::to_vec(&msg.header).ok()?);

    // offset_2: parent_header
    offsets.push(offset_0 + payload.len() as u64);
    payload.append(&mut serde_json::to_vec(&msg.parent_header).ok()?);

    // offset_3: metadata
    offsets.push(offset_0 + payload.len() as u64);
    payload.extend_from_slice(b"{}");

    // offset_4: content
    offsets.push(offset_0 + payload.len() as u64);
    payload.append(&mut serde_json::to_vec(&msg.content).ok()?);

    for buffer in &msg.buffers {
        offsets.push(offset_0 + payload.len() as u64);
        payload.extend_from_slice(buffer);
    }

    Some(
        offsets
            .into_iter()
            .flat_map(|n| n.to_le_bytes())
            .chain(payload)
            .collect::<Vec<u8>>(),
    )
}

fn from_ws_payload(payload: &[u8]) -> Option<(KernelMessage, String)> {
    let offset_number: usize = u64::from_le_bytes(payload.get(0..8)?.try_into().ok()?)
        .try_into()
        .ok()?;

    let mut offsets = Vec::with_capacity(offset_number);
    for i in 0..offset_number {
        let index = 8 * (i + 1);
        offsets.push(
            u64::from_le_bytes(payload.get(index..index + 8)?.try_into().ok()?)
                .try_into()
                .ok()?,
        );
    }
    offsets.push(payload.len());

    let channel = String::from_utf8(payload.get(offsets[0]..offsets[1])?.to_vec()).ok()?;
    let header = serde_json::from_slice(payload.get(offsets[1]..offsets[2])?).ok()?;
    let parent_header = serde_json::from_slice(payload.get(offsets[2]..offsets[3])?).ok()?;
    // serde_json::from_slice(payload.get(offsets[3]..offsets[4])?).ok()?;
    let content = serde_json::from_slice(payload.get(offsets[4]..offsets[5])?).ok()?;

    let mut buffers = Vec::new();
    for i in 5..offset_number {
        buffers.push(Bytes::from(
            payload.get(offsets[i]..offsets[i + 1])?.to_vec(),
        ));
    }

    let msg = KernelMessage {
        header,
        parent_header,
        content,
        buffers,
    };
    Some((msg, channel))
}

/// Connect to Jupyter via the `v1.kernel.websocket.jupyter.org` protocol.
pub async fn create_websocket_connection(
    websocket_url: &str,
    token: &str,
) -> Result<KernelConnection, Error> {
    let (shell_tx, shell_rx) = async_channel::bounded(8);
    let (control_tx, control_rx) = async_channel::bounded(8);
    let (iopub_tx, iopub_rx) = async_channel::bounded(64);
    let reply_tx_map = Arc::new(DashMap::new());
    let signal = CancellationToken::new();

    let conn = KernelConnection {
        shell_tx,
        control_tx,
        iopub_rx,
        reply_tx_map: reply_tx_map.clone(),
        signal: signal.clone(),
        _drop_guard: Arc::new(signal.clone().drop_guard()),
    };

    let mut req = websocket_url
        .into_client_request()
        .map_err(|err| Error::KernelConnect(err.to_string()))?;

    req.headers_mut().insert(
        SEC_WEBSOCKET_PROTOCOL,
        HeaderValue::from_static("v1.kernel.websocket.jupyter.org"),
    );
    req.headers_mut().insert(
        AUTHORIZATION,
        format!("token {token}")
            .parse::<HeaderValue>()
            .map_err(|err| Error::KernelConnect(err.to_string()))?,
    );

    let (ws, _resp) = tokio_tungstenite::connect_async(req)
        .await
        .map_err(|err| Error::KernelConnect(err.to_string()))?;

    let (mut ws_tx, mut ws_rx) = ws.split();
    let send_fut = async move {
        // Send shell and control messages over the WebSocket.
        loop {
            let (msg, channel) = tokio::select! {
                Ok(msg) = shell_rx.recv() => (msg, "shell"),
                Ok(msg) = control_rx.recv() => (msg, "control"),
                else => break,
            };

            let Some(payload) = to_ws_payload(&msg, channel) else {
                error!("error converting message to ws payload");
                continue;
            };

            if ws_tx.send(Message::Binary(payload)).await.is_err() {
                // The WebSocket has been closed.
                // TODO: Handle reconnection.
                error!("WebSocket closed, reconnection not yet implemented");
                break;
            }
        }
    };

    let receive_fut = async move {
        // Receieve shell, control, and iopub messages from the WebSocket.
        while let Some(Ok(ws_payload)) = ws_rx.next().await {
            let payload = match ws_payload {
                Message::Binary(payload) => payload,
                _ => continue,
            };

            let (msg, channel) = match from_ws_payload(&payload) {
                Some(msg) => msg,
                None => continue,
            };

            match &*channel {
                "shell" | "control" => {
                    if let Some(KernelHeader { msg_id, .. }) = &msg.parent_header {
                        if let Some((_, tx)) = reply_tx_map.remove(msg_id) {
                            // Optional, it's not an error if this receiver has been dropped.
                            _ = tx.send(msg);
                        }
                    }
                }
                "iopub" => {
                    _ = iopub_tx.send(msg).await;
                }
                _ => {
                    warn!("received WebSocket message on unexpected channel: {channel}");
                }
            }
        }
    };

    // Run both futures until cancellation or completion.
    tokio::spawn(async move {
        tokio::select! {
            _ = async { tokio::join!(send_fut, receive_fut) } => {}
            _ = signal.cancelled() => {}
        }
    });

    Ok(conn)
}
