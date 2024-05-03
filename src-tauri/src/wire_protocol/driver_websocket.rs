use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use reqwest::header::SEC_WEBSOCKET_PROTOCOL;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use tracing::warn;

use super::{KernelConnection, KernelMessage};
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

fn to_ws_payload(msg: &KernelMessage<serde_json::Value>, channel: &str) -> Option<Vec<u8>> {
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

fn from_ws_payload(payload: &[u8]) -> Option<(KernelMessage<serde_json::Value>, String)> {
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
pub async fn create_websocket_connection(websocket_url: &str) -> Result<KernelConnection, Error> {
    let (shell_tx, shell_rx) = async_channel::bounded(1);
    let (control_tx, control_rx) = async_channel::bounded(1);
    let (iopub_tx, iopub_rx) = async_channel::bounded(1);

    let conn = KernelConnection {
        shell_tx,
        control_tx,
        iopub_rx,
    };

    let mut req = websocket_url
        .into_client_request()
        .map_err(|err| Error::KernelConnect(err.to_string()))?;
    req.headers_mut().insert(
        SEC_WEBSOCKET_PROTOCOL,
        "v1.kernel.websocket.jupyter.org".parse().unwrap(),
    );

    let (ws, _resp) = tokio_tungstenite::connect_async(req)
        .await
        .map_err(|err| Error::KernelConnect(err.to_string()))?;

    let (mut ws_tx, mut ws_rx) = ws.split();
    tokio::spawn(async move {
        // Send shell and control messages over the WebSocket.
        loop {
            let (msg, channel) = tokio::select! {
                Ok(msg) = shell_rx.recv() => (msg, "shell"),
                Ok(msg) = control_rx.recv() => (msg, "control"),
                else => break,
            };

            let Some(payload) = to_ws_payload(&msg, channel) else {
                break;
            };

            if ws_tx.send(Message::Binary(payload)).await.is_err() {
                // The WebSocket has been closed.
                // TODO: Handle reconnection.
                break;
            }
        }
    });

    tokio::spawn(async move {
        // Receieve iopub messages from the WebSocket.
        while let Some(Ok(ws_payload)) = ws_rx.next().await {
            let payload = match ws_payload {
                Message::Binary(payload) => payload,
                _ => continue,
            };

            let (msg, channel) = match from_ws_payload(&payload) {
                Some(msg) => msg,
                None => continue,
            };
            if channel != "iopub" {
                warn!("received WebSocket message on unexpected channel: {channel}");
            }

            if iopub_tx.send(msg).await.is_err() {
                // The receiver has been dropped.
                break;
            }
        }
    });

    Ok(conn)
}
