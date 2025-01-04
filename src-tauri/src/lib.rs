//! Library code for the Jute application.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::fmt;
use std::io;

pub mod backend;
pub mod plugins;
pub mod state;

// The error we return from the application.
#[derive(Debug, Clone, thiserror::Error, serde::Serialize, specta::Type)]
pub struct ErrorResponse {
    // The error message.
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

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
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl Error {
    /// Translates the error into an ErrorResponse with a basic message.
    pub fn as_response(&self) -> ErrorResponse {
        ErrorResponse {
            message: match self {
                Error::Subprocess(err) => format!("Failed to run subprocess: {}", err),
                Error::KernelConnect(msg) => format!("Could not connect to the kernel: {}", msg),
                Error::KernelDisconnect => "Disconnected from the kernel".to_string(),
                Error::InvalidUrl(err) => format!("Invalid URL: {}", err),
                Error::ReqwestError(err) => format!("HTTP failure: {}", err),
                Error::DeserializeMessage(msg) => format!("Could not deserialize message: {}", msg),
                Error::Zmq(err) => format!("ZeroMQ error: {}", err),
            },
        }
    }
}

impl From<Error> for ErrorResponse {
    fn from(error: Error) -> Self {
        error.as_response()
    }
}
