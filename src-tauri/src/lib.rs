//! Library code for the Jute application.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::io;

pub mod backend;
pub mod entity;
pub mod menu;
pub mod plugins;
pub mod state;
pub mod window;

/// A serializable error type for application errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while starting or managing a subprocess.
    #[error("failed to run subprocess: {0}")]
    Subprocess(io::Error),

    /// Could not connect to the kernel.
    #[error("could not connect to the kernel: {0}")]
    KernelConnect(String),

    /// Disconnected while communicating with a kernel.
    #[error("disconnected from the kernel")]
    KernelDisconnect,

    /// An invalid URL was provided or constructed.
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// HTTP error from reqwest while making a request.
    #[error("HTTP failure: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Error while deserializing a message.
    #[error("could not deserialize message: {0}")]
    DeserializeMessage(String),

    /// Error originating from ZeroMQ.
    #[error("zeromq: {0}")]
    Zmq(#[from] zeromq::ZmqError),

    /// Error originating from serde_json.
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::error::Error),

    /// Error interacting with the filesystem.
    #[error("filesystem error: {0}")]
    Filesystem(io::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
