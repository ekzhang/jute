use bytes::Bytes;
use tracing::{error, warn};
use zeromq::{Socket, SocketRecv, SocketSend, ZmqMessage};

use super::{KernelConnection, KernelMessage};
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

fn to_zmq_payload(msg: &KernelMessage<serde_json::Value>, signing_key: &str) -> Option<ZmqMessage> {
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

fn from_zmq_payload(payload: ZmqMessage) -> Option<KernelMessage<serde_json::Value>> {
    let payload = payload.into_vec();

    let delim_idx = payload.iter().position(|b| *b == b"<IDS|MSG>" as &[u8])?;
    let header = serde_json::from_slice(&payload[delim_idx + 2]).ok()?;
    let parent_header = serde_json::from_slice(&payload[delim_idx + 3]).ok()?;
    serde_json::from_slice(&payload[delim_idx + 4]).ok()?; // metadata
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
    let (shell_tx, shell_rx) = async_channel::bounded(1);
    let (control_tx, control_rx) = async_channel::bounded(1);
    let (iopub_tx, iopub_rx) = async_channel::bounded(1);

    let conn = KernelConnection {
        shell_tx,
        control_tx,
        iopub_rx,
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
    tokio::spawn(async move {
        // Send shell messages.
        while let Ok(msg) = shell_rx.recv().await {
            let Some(payload) = to_zmq_payload(&msg, &key) else {
                error!("error converting shell message to zmq payload");
                break;
            };
            if let Err(err) = shell.send(payload).await {
                warn!("error sending zmq shell message: {err:?}");
            }
        }
    });

    let key = signing_key.to_string();
    tokio::spawn(async move {
        // Send control messages.
        while let Ok(msg) = control_rx.recv().await {
            let Some(payload) = to_zmq_payload(&msg, &key) else {
                error!("error converting control message to zmq payload");
                break;
            };
            if let Err(err) = control.send(payload).await {
                warn!("error sending zmq control message: {err:?}");
            }
        }
    });

    tokio::spawn(async move {
        // Receive iopub messages.
        while let Ok(payload) = iopub.recv().await {
            if let Some(msg) = from_zmq_payload(payload) {
                if iopub_tx.send(msg).await.is_err() {
                    break;
                }
            } else {
                warn!("error converting zmq payload to iopub message");
            }
        }
    });

    Ok(conn)
}