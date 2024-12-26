//! Backend for Jute, connecting to local Jupyter kernels or remote servers.
//!
//! The local and remote kernels have a shared wire protocol, so that lives
//! outside either folder.

pub use wire_protocol::{create_websocket_connection, create_zeromq_connection, KernelConnection};

pub mod commands;
pub mod local;
pub mod remote;
pub mod wire_protocol;
