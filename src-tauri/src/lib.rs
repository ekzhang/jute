//! Library code for the Jute application.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::io;

pub mod kernel_client;
pub mod server;

/// A serializable error type for application errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while starting a subprocess.
    #[error("subprocess failed to start: {0}")]
    Subprocess(io::Error),

    /// Disconnected while communicating with a kernel.
    #[error("disconnected from the kernel")]
    KernelDisconnect,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
