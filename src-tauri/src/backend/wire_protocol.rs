//! Jupyter kernel wire protocol implementations.
//!
//! See the [Messaging in Jupyter](https://jupyter-client.readthedocs.io/en/stable/messaging.html)
//! page for documentation about how this works. The wire protocol is used to
//! communicate with Jupyter kernels over ZeroMQ or WebSocket.

use std::collections::BTreeMap;
use std::sync::Arc;

use bytes::Bytes;
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::oneshot;
use tokio_util::sync::{CancellationToken, DropGuard};
use ts_rs::TS;
use uuid::Uuid;

pub use self::driver_websocket::create_websocket_connection;
pub use self::driver_zeromq::create_zeromq_connection;
use crate::Error;

mod driver_websocket;
mod driver_zeromq;

/// Type of a kernel wire protocol message, either request or reply.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum KernelMessageType {
    /// Execute a block of code.
    ExecuteRequest,

    /// Return execution results.
    ExecuteReply,

    /// Request detailed information about a piece of code.
    InspectRequest,

    /// Return detailed information about the inspected code.
    InspectReply,

    /// Request code completions or suggestions.
    CompleteRequest,

    /// Return completions or suggestions for the code.
    CompleteReply,

    /// Request execution history (not often used).
    HistoryRequest,

    /// Return the requested execution history (not often used).
    HistoryReply,

    /// Request to check if code is complete.
    IsCompleteRequest,

    /// Reply indicating if code is complete.
    IsCompleteReply,

    /// Request information about existing comms.
    CommInfoRequest,

    /// Reply with information about existing comms.
    CommInfoReply,

    /// Request kernel information.
    KernelInfoRequest,

    /// Reply with kernel information.
    KernelInfoReply,

    /// Request kernel shutdown.
    ShutdownRequest,

    /// Reply to confirm kernel shutdown.
    ShutdownReply,

    /// Request to interrupt kernel execution.
    InterruptRequest,

    /// Reply to confirm kernel interruption.
    InterruptReply,

    /// Request to start or stop a debugger.
    DebugRequest,

    /// Reply with debugger status.
    DebugReply,

    /// Streams of output (stdout, stderr) from the kernel.
    Stream,

    /// Bring back data to be displayed in frontends.
    DisplayData,

    /// Update display data with new information.
    UpdateDisplayData,

    /// Re-broadcast of code in ExecuteRequest.
    ExecuteInput,

    /// Results of a code execution.
    ExecuteResult,

    /// When an error occurs during code execution.
    Error,

    /// Updates about kernel status.
    Status,

    /// Clear output visible on the frontend.
    ClearOutput,

    /// For debugging kernels to send events.
    DebugEvent,

    /// Another kernel message type that is unrecognized.
    #[serde(untagged)]
    Other(String),
}

/// Header of a message, generally part of the {header, parent_header, metadata,
/// content, buffers} 5-tuple.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct KernelHeader {
    /// Typically UUID, must be unique per message.
    pub msg_id: String,

    /// Typically UUID, should be unique per session.
    pub session: String,

    /// The username of the user sending the message.
    pub username: String,

    /// ISO 8601 timestamp for when the message is created.
    #[serde(with = "time::serde::iso8601")]
    #[ts(type = "string")]
    pub date: OffsetDateTime,

    /// The message type.
    pub msg_type: KernelMessageType,

    /// Message protocol version.
    pub version: String,
}

/// A message sent to or received from a Jupyter kernel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelMessage<T = serde_json::Value> {
    /// The message header.
    pub header: KernelHeader,

    /// The parent message header, if any.
    pub parent_header: Option<KernelHeader>,

    /// The content of the message.
    pub content: T,

    /// Buffers for large data, if any (used by extensions).
    pub buffers: Vec<Bytes>,
}

impl<T> KernelMessage<T> {
    /// Create a basic kernel message with the given header and content.
    pub fn new(msg_type: KernelMessageType, content: T) -> Self {
        Self {
            header: KernelHeader {
                msg_id: Uuid::new_v4().to_string(),
                session: "jute-session".to_string(),
                username: "jute-user".to_string(),
                date: OffsetDateTime::now_utc(),
                msg_type,
                version: "5.4".into(),
            },
            parent_header: None,
            content,
            buffers: Vec::new(),
        }
    }
}

impl<T: Serialize> KernelMessage<T> {
    /// Produce a variant of the message as a serialized JSON type.
    pub fn into_json(self) -> KernelMessage {
        KernelMessage {
            header: self.header,
            parent_header: self.parent_header,
            content: serde_json::to_value(&self.content).expect("KernelMessage JSON serialization"),
            buffers: self.buffers,
        }
    }
}

impl KernelMessage {
    /// Deserialize the content of the message into a specific type.
    pub fn into_typed<T: DeserializeOwned>(self) -> Result<KernelMessage<T>, Error> {
        Ok(KernelMessage {
            header: self.header,
            parent_header: self.parent_header,
            content: serde_json::from_value(self.content)
                .map_err(|err| Error::DeserializeMessage(err.to_string()))?,
            buffers: self.buffers,
        })
    }
}

/// The content of a reply to a kernel message, with status attached.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Reply<T> {
    /// The request was processed successfully.
    Ok(T),

    /// The request failed due to an error.
    Error(ErrorReply),

    /// This is the same as `status="error"` but with no information about the
    /// error. No fields should be present other than status.
    ///
    /// Some messages like execute_reply return "aborted" instead, see
    /// <https://github.com/ipython/ipykernel/issues/367> for details.
    #[serde(alias = "aborted")]
    Abort,
}

/// Content of an error response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct ErrorReply {
    /// The error name, such as 'NameError'.
    pub ename: String,

    /// The error message, such as 'NameError: name 'x' is not defined'.
    pub evalue: String,

    /// The traceback frames of the error as a list of strings.
    pub traceback: Vec<String>,
}

/// Execute code on behalf of the user.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct ExecuteRequest {
    /// Source code to be executed by the kernel, one or more lines.
    pub code: String,

    /// A boolean flag which, if true, signals the kernel to execute the code as
    /// quietly as possible.
    pub silent: bool,

    /// A boolean flag which, if true, signals the kernel to populate the
    /// history.
    pub store_history: bool,

    /// A dictionary mapping names to expressions to be evaluated in the user's
    /// dictionary. The rich display-data representation of each will be
    /// evaluated after execution.
    pub user_expressions: BTreeMap<String, String>,

    /// If true, code running in the kernel can prompt the user for input with
    /// an `input_request` message. If false, the kernel should not send
    /// these messages.
    pub allow_stdin: bool,

    /// A boolean flag, which, if true, aborts the execution queue if an
    /// exception is encountered. If false, queued `execute_requests` will
    /// execute even if this request generates an exception.
    pub stop_on_error: bool,
}

/// Represents a reply to an execute request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct ExecuteReply {
    /// The execution count, which increments with each request that stores
    /// history.
    pub execution_count: i32,

    /// Results for the user expressions evaluated during execution. Only
    /// present when status is 'ok'.
    pub user_expressions: Option<BTreeMap<String, String>>,
}

/// Request for introspection of code to retrieve useful information as
/// determined by the kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct InspectRequest {
    /// The code context in which introspection is requested, potentially
    /// multiple lines.
    pub code: String,

    /// The cursor position within 'code' where introspection is requested, in
    /// Unicode characters.
    pub cursor_pos: u32,

    /// The level of detail desired, where 0 might be basic info (`x?` in
    /// IPython) and 1 includes more detail (`x??` in IPython).
    pub detail_level: u8,
}

/// Represents a reply to an inspect request with potentially formatted
/// information about the code context.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct InspectReply {
    /// Indicates whether an object was found during the inspection.
    pub found: bool,

    /// A dictionary containing the data representing the inspected object, can
    /// be empty if nothing is found.
    pub data: BTreeMap<String, serde_json::Value>,

    /// Metadata associated with the data, can also be empty.
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Request for code completion based on the context provided in the code and
/// cursor position.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct CompleteRequest {
    /// The code context in which completion is requested, possibly a multiline
    /// string.
    pub code: String,

    /// The cursor position within 'code' in Unicode characters where completion
    /// is requested.
    pub cursor_pos: u32,
}

/// Represents a reply to a completion request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct CompleteReply {
    /// A list of all matches to the completion request.
    pub matches: Vec<String>,

    /// The starting position of the text that should be replaced by the
    /// completion.
    pub cursor_start: u32,

    /// The ending position of the text that should be replaced by the
    /// completion.
    pub cursor_end: u32,

    /// Metadata providing additional information about completions.
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Request for information about the kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct KernelInfoRequest {}

/// Represents a reply to a kernel_info request, providing details about the
/// kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct KernelInfoReply {
    /// Version of the messaging protocol used by the kernel.
    pub protocol_version: String,

    /// The name of the kernel implementation (e.g., 'ipython').
    pub implementation: String,

    /// The version number of the kernel's implementation.
    pub implementation_version: String,

    /// Detailed information about the programming language used by the kernel.
    pub language_info: LanguageInfo,

    /// A banner of information about the kernel, dispalyed in console.
    pub banner: String,

    /// Indicates if the kernel supports debugging.
    #[serde(default)]
    pub debugger: bool,
}

/// Detailed information about the programming language of the kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct LanguageInfo {
    /// Name of the programming language.
    pub name: String,

    /// Version number of the language.
    pub version: String,

    /// MIME type for script files in this language.
    pub mimetype: String,

    /// File extension for script files in this language.
    pub file_extension: String,

    /// Nbconvert exporter, if notebooks should be exported differently than the
    /// general script.
    pub nbconvert_exporter: String,
}

/// Request to shut down the kernel, possibly to prepare for a restart.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct ShutdownRequest {
    /// Indicates whether the shutdown is final or precedes a restart.
    pub restart: bool,
}

/// Represents a reply to a shutdown request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct ShutdownReply {
    /// Matches the restart flag from the request to indicate the intended
    /// shutdown behavior.
    pub restart: bool,
}

/// Request to interrupt the kernel's current operation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct InterruptRequest {}

/// Represents a reply to an interrupt request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct InterruptReply {}

/// Streams of output from the kernel, such as stdout and stderr.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct Stream {
    /// The name of the stream, one of 'stdout' or 'stderr'.
    pub name: String,

    /// The text to be displayed in the stream.
    pub text: String,
}

/// Data to be displayed in frontends, such as images or HTML.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct DisplayData {
    /// The data to be displayed, typically a MIME type and the data itself.
    pub data: BTreeMap<String, serde_json::Value>,

    /// Metadata associated with the data, can be empty.
    pub metadata: BTreeMap<String, serde_json::Value>,

    /// Any information not to be persisted to a notebook.
    pub transient: Option<DisplayDataTransient>,
}

/// Transient data associated with display data, such as display IDs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct DisplayDataTransient {
    /// Specifies an ID for the display, which can be updated.
    pub display_id: Option<String>,
}

/// Re-broadcast of code in an execute request to let all frontends know.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct ExecuteInput {
    /// The code that was executed.
    pub code: String,

    /// The execution count, which increments with each request that stores
    /// history.
    pub execution_count: i32,
}

/// Results of a code execution, such as the output or return value.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct ExecuteResult {
    /// The execution count, which increments with each request that stores
    /// history.
    pub execution_count: i32,

    /// The data to be displayed, typically a MIME type and the data itself. A
    /// plain text representation should always be provided in the `text/plain`
    /// mime-type.
    pub data: BTreeMap<String, serde_json::Value>,

    /// Metadata associated with the data, can be empty.
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Used by frontends to monitor the status of the kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct Status {
    /// Current status of the kernel.
    pub execution_state: KernelStatus,
}

/// Possible states of the kernel. When the kernel starts to handle a message,
/// it will enter the 'busy' state and when it finishes, it will enter the
/// 'idle' state. The kernel will publish state 'starting' exactly once at
/// process startup.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
pub enum KernelStatus {
    /// The kernel is starting up.
    Starting,

    /// The kernel is ready to execute code.
    Idle,

    /// The kernel is currently executing code.
    Busy,
}

/// Request to clear output visible on the frontend.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
pub struct ClearOutput {
    /// The wait flag, which if true, indicates that the frontend should wait
    /// for the clear output request to complete before sending further
    /// messages.
    pub wait: bool,
}

/// Represents a stateful kernel connection that can be used to communicate with
/// a running Jupyter kernel.
///
/// Connections can be obtained through either WebSocket or ZeroMQ network
/// protocols. They send messages to the kernel and receive responses through
/// one of the five dedicated channels:
///
/// - Shell: Main channel for code execution and info requests.
/// - IOPub: Broadcast channel for side effects (stdout, stderr) and requests
///   from any client over the shell channel.
/// - Stdin: Requests from the kernel to the client for standard input.
/// - Control: Just like Shell, but separated to avoid queueing.
/// - Heartbeat: Periodic ping/pong to ensure the connection is alive. This
///   appears to only be supported by ZeroMQ, so we don't implement it here.
///
/// The specific details of which messages are sent on which channels are left
/// to the user. Functions will block if disconnected or return an error after
/// the driver has been closed.
#[derive(Clone)]
pub struct KernelConnection {
    shell_tx: async_channel::Sender<KernelMessage>,
    control_tx: async_channel::Sender<KernelMessage>,
    iopub_rx: async_channel::Receiver<KernelMessage>,
    reply_tx_map: Arc<DashMap<String, oneshot::Sender<KernelMessage>>>,
    signal: CancellationToken,
    _drop_guard: Arc<DropGuard>,
}

impl KernelConnection {
    /// Send a message to the kernel over the shell channel.
    ///
    /// On success, return a receiver for the reply from the kernel on the same
    /// channel, when it is finished.
    pub async fn call_shell<T: Serialize>(
        &self,
        message: KernelMessage<T>,
    ) -> Result<PendingRequest, Error> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg_id = message.header.msg_id.clone();
        self.reply_tx_map.insert(msg_id.clone(), reply_tx);

        self.shell_tx
            .send(message.into_json())
            .await
            .map_err(|_| Error::KernelDisconnect)?;

        Ok(PendingRequest {
            reply_tx_map: self.reply_tx_map.clone(),
            reply_rx,
            msg_id,
        })
    }

    /// Send a message to the kernel over the control channel.
    pub async fn call_control<T: Serialize>(
        &self,
        message: KernelMessage<T>,
    ) -> Result<PendingRequest, Error> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg_id = message.header.msg_id.clone();
        self.reply_tx_map.insert(msg_id.clone(), reply_tx);

        self.control_tx
            .send(message.into_json())
            .await
            .map_err(|_| Error::KernelDisconnect)?;

        Ok(PendingRequest {
            reply_tx_map: self.reply_tx_map.clone(),
            reply_rx,
            msg_id,
        })
    }

    /// Receieve a message from the kernel over the iopub channel.
    pub async fn recv_iopub(&self) -> Result<KernelMessage, Error> {
        self.iopub_rx
            .recv()
            .await
            .map_err(|_| Error::KernelDisconnect)
    }

    /// Receive an immediate message over the iopub channel without waiting.
    pub fn try_recv_iopub(&self) -> Option<KernelMessage> {
        self.iopub_rx.try_recv().ok()
    }

    /// Close the connection to the kernel, shutting down all channels.
    pub fn close(&self) {
        self.shell_tx.close();
        self.control_tx.close();
        self.iopub_rx.close();
        self.signal.cancel(); // This is the only necessary line, but we close
                              // the channels for good measure regardless.
    }
}

/// Receives a reply from a previous kernel router-dealer request.
pub struct PendingRequest {
    reply_tx_map: Arc<DashMap<String, oneshot::Sender<KernelMessage>>>,
    reply_rx: oneshot::Receiver<KernelMessage>,
    msg_id: String,
}

impl PendingRequest {
    /// Wait for the reply to the previous request from the kernel.
    pub async fn get_reply<U: DeserializeOwned>(
        &mut self,
    ) -> Result<KernelMessage<Reply<U>>, Error> {
        (&mut self.reply_rx)
            .await
            .map_err(|_| Error::KernelDisconnect)?
            .into_typed()
    }
}

impl Drop for PendingRequest {
    fn drop(&mut self) {
        // This ensures that we don't leak memory by leaving the channel in the map.
        self.reply_tx_map.remove(&self.msg_id);
    }
}
