//! Adapter for the Jupyter wire protocol over ZeroMQ.
//!
//! This protocol is documented in the `jupyter-client` project at
//! <https://jupyter-client.readthedocs.io/en/stable/messaging.html>. It relies
//! on 5 dedicated sockets for different types of messages.

use std::sync::Arc;

use bytes::Bytes;
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};
use zeromq::{Socket, SocketRecv, SocketSend, ZmqMessage};

use super::{KernelConnection, KernelHeader, KernelMessage};
use crate::Error;

/// Sign a message using HMAC-SHA256 with the kernel's signing key.
fn sign_message(signing_key: &str, bytes: &[Bytes]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(signing_key.as_bytes()).unwrap();
    for b in bytes {
        mac.update(b);
    }
    format!("{:x}", mac.finalize().into_bytes())
}

fn to_zmq_payload(msg: &KernelMessage, signing_key: &str) -> Option<ZmqMessage> {
    let header = Bytes::from(serde_json::to_vec(&msg.header).ok()?);
    let parent_header = Bytes::from(serde_json::to_vec(&msg.parent_header).ok()?);
    let metadata = Bytes::from_static(b"{}");
    let content = Bytes::from(serde_json::to_vec(&msg.content).ok()?);

    let mut payload = vec![header, parent_header, metadata, content];
    payload.extend(msg.buffers.iter().cloned());

    let signature = sign_message(signing_key, &payload);
    payload.insert(0, Bytes::from(signature));
    payload.insert(0, Bytes::from_static(b"<IDS|MSG>"));

    ZmqMessage::try_from(payload).ok()
}

fn from_zmq_payload(payload: ZmqMessage) -> Option<KernelMessage> {
    let payload = payload.into_vec();

    let delim_idx = payload.iter().position(|b| *b == b"<IDS|MSG>" as &[u8])?;
    let header = serde_json::from_slice(&payload[delim_idx + 2]).ok()?;
    let parent_header = serde_json::from_slice(&payload[delim_idx + 3]).ok()?;
    // serde_json::from_slice(&payload[delim_idx + 4]).ok()?;
    let content = serde_json::from_slice(&payload[delim_idx + 5]).ok()?;
    let buffers = payload[delim_idx + 6..].to_vec();

    Some(KernelMessage {
        header,
        parent_header,
        content,
        buffers,
    })
}

/// Connect to Jupyter via ZeroMQ to a local kernel.
pub async fn create_zeromq_connection(
    shell_port: u16,
    control_port: u16,
    iopub_port: u16,
    stdin_port: u16,
    heartbeat_port: u16,
    signing_key: &str,
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

    let mut shell = zeromq::DealerSocket::new();
    shell
        .connect(&format!("tcp://127.0.0.1:{shell_port}"))
        .await?;
    let mut control = zeromq::DealerSocket::new();
    control
        .connect(&format!("tcp://127.0.0.1:{control_port}"))
        .await?;
    let mut iopub = zeromq::SubSocket::new();
    iopub
        .connect(&format!("tcp://127.0.0.1:{iopub_port}"))
        .await?;
    iopub.subscribe("").await?;
    let mut stdin = zeromq::DealerSocket::new();
    stdin
        .connect(&format!("tcp://127.0.0.1:{stdin_port}"))
        .await?;
    let mut heartbeat = zeromq::ReqSocket::new();
    heartbeat
        .connect(&format!("tcp://127.0.0.1:{heartbeat_port}"))
        .await?;

    let _ = (stdin, heartbeat); // Not supported yet.

    let key = signing_key.to_string();
    let tx_map = reply_tx_map.clone();
    let shell_fut = async move {
        // Send and receive shell messages.
        loop {
            tokio::select! {
                Ok(msg) = shell_rx.recv() => {
                    let Some(payload) = to_zmq_payload(&msg, &key) else {
                        error!("error converting shell message to zmq payload");
                        continue;
                    };
                    if let Err(err) = shell.send(payload).await {
                        warn!("error sending zmq shell message: {err:?}");
                    }
                }
                Ok(payload) = shell.recv() => {
                    if let Some(msg) = from_zmq_payload(payload) {
                        if let Some(KernelHeader { msg_id, .. }) = &msg.parent_header {
                            if let Some((_, reply_tx)) = tx_map.remove(msg_id) {
                                _ = reply_tx.send(msg);
                            }
                        }
                    } else {
                        warn!("error converting zmq payload to shell reply");
                    }
                }
                else => break,
            }
        }
    };

    let key = signing_key.to_string();
    let tx_map = reply_tx_map.clone();
    let control_fut = async move {
        // Send and receive control messages.
        loop {
            tokio::select! {
                Ok(msg) = control_rx.recv() => {
                    let Some(payload) = to_zmq_payload(&msg, &key) else {
                        error!("error converting control message to zmq payload");
                        continue;
                    };
                    if let Err(err) = control.send(payload).await {
                        warn!("error sending zmq control message: {err:?}");
                    }
                }
                Ok(payload) = control.recv() => {
                    if let Some(msg) = from_zmq_payload(payload) {
                        if let Some(KernelHeader { msg_id, .. }) = &msg.parent_header {
                            if let Some((_, reply_tx)) = tx_map.remove(msg_id) {
                                _ = reply_tx.send(msg);
                            }
                        }
                    } else {
                        warn!("error converting zmq payload to control reply");
                    }
                }
                else => break,
            }
        }
    };

    let iopub_fut = async move {
        // Receive iopub messages.
        while let Ok(payload) = iopub.recv().await {
            if let Some(msg) = from_zmq_payload(payload) {
                _ = iopub_tx.send(msg).await;
            } else {
                warn!("error converting zmq payload to iopub message");
            }
        }
    };

    tokio::spawn(async move {
        tokio::select! {
            _ = async { tokio::join!(shell_fut, control_fut, iopub_fut) } => {}
            _ = signal.cancelled() => {}
        }
    });

    Ok(conn)
}
