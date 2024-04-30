//! Jupyter kernel wire protocol implementations.
//!
//! See the [Messaging in Jupyter](https://jupyter-client.readthedocs.io/en/stable/messaging.html)
//! page for documentation about how this works. The wire protocol is used to
//! communicate with Jupyter kernels over ZeroMQ or WebSocket.

use std::collections::BTreeMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Type of a kernel wire protocol message, either request or reply.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
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
}

/// Version of wire protocol being used. Only version 5.4 is used and supported.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum KernelProtocolVersion {
    /// Version 5.4.
    #[serde(rename = "5.4")]
    V5_4,
}

/// Header of a message, generally part of the {header, parent_header, metadata,
/// content, buffers} 5-tuple.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KernelHeader {
    /// Typically UUID, must be unique per message.
    pub msg_id: String,

    /// Typically UUID, should be unique per session.
    pub session: String,

    /// The username of the user sending the message.
    pub username: String,

    /// ISO 8601 timestamp for when the message is created.
    #[serde(with = "time::serde::iso8601")]
    pub date: OffsetDateTime,

    /// The message type.
    pub msg_type: KernelMessageType,

    /// Message protocol version.
    pub version: KernelProtocolVersion,
}

/// A message sent to or received from a Jupyter kernel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelMessage<T> {
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
    pub fn new(msg_type: KernelMessageType, session: String, content: T) -> Self {
        Self {
            header: KernelHeader {
                msg_id: Uuid::new_v4().to_string(),
                session,
                username: "jute".to_string(),
                date: OffsetDateTime::now_utc(),
                msg_type,
                version: KernelProtocolVersion::V5_4,
            },
            parent_header: None,
            content,
            buffers: Vec::new(),
        }
    }
}

/// Execute code on behalf of the user.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExecuteReply {
    /// The status of the execution, can be 'ok', 'error', or 'aborted'.
    pub status: String,

    /// The execution count, which increments with each request that stores
    /// history.
    pub execution_count: i32,

    /// Results for the user expressions evaluated during execution. Only
    /// present when status is 'ok'.
    pub user_expressions: Option<BTreeMap<String, String>>,
}

/// Request for introspection of code to retrieve useful information as
/// determined by the kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InspectReply {
    /// The status of the inspection, 'ok' if successful, 'error' otherwise.
    pub status: String,

    /// Indicates whether an object was found during the inspection.
    pub found: bool,

    /// A dictionary containing the data representing the inspected object, can
    /// be empty if nothing is found.
    pub data: BTreeMap<String, String>,

    /// Metadata associated with the data, can also be empty.
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Request for code completion based on the context provided in the code and
/// cursor position.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CompleteRequest {
    /// The code context in which completion is requested, possibly a multiline
    /// string.
    pub code: String,

    /// The cursor position within 'code' in Unicode characters where completion
    /// is requested.
    pub cursor_pos: u32,
}

/// Represents a reply to a completion request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CompleteReply {
    /// The status of the completion, 'ok' if successful, 'error' otherwise.
    pub status: String,

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

/// Request to determine if the provided code is complete and ready for
/// execution.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IsCompleteRequest {
    /// The code entered so far, possibly spanning multiple lines.
    pub code: String,
}

/// Represents a reply to an is_complete request, indicating the completeness
/// status of the code.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IsCompleteReply {
    /// The status of the code's completeness: 'complete', 'incomplete',
    /// 'invalid', or 'unknown'.
    pub status: String,

    /// Suggested characters to indent the next line if the code is incomplete.
    /// This field is optional and used only if the status is 'incomplete'.
    pub indent: Option<String>,
}

/// Request for information about the kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KernelInfoRequest {}

/// Represents a reply to a kernel_info request, providing details about the
/// kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KernelInfoReply {
    /// The status of the request, 'ok' if successful, 'error' otherwise.
    pub status: String,

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
    pub debugger: bool,
}

/// Detailed information about the programming language of the kernel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ShutdownRequest {
    /// Indicates whether the shutdown is final or precedes a restart.
    pub restart: bool,
}

/// Represents a reply to a shutdown request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ShutdownReply {
    /// The status of the shutdown request, 'ok' if successful, 'error'
    /// otherwise.
    pub status: String,

    /// Matches the restart flag from the request to indicate the intended
    /// shutdown behavior.
    pub restart: bool,
}

/// Request to interrupt the kernel's current operation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InterruptRequest {}

/// Represents a reply to an interrupt request.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InterruptReply {
    /// The status of the interrupt request, 'ok' if successful, 'error'
    /// otherwise.
    pub status: String,
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
/// - Heartbeat: Periodic ping/pong to ensure the connection is alive.
///
/// The specific details of which messages are sent on which channels are left
/// to the user of this trait.
pub trait KernelConnection {
    // TODO
}

/// Connection to Jupyter via the `v1.kernel.websocket.jupyter.org` protocol.
pub struct KernelWebSocketConnection {}

/// Connection to Jupyter via ZeroMQ to a local kernel.
pub struct KernelZmqConnection {}
