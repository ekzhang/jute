//! Defines state and stores for the Tauri application.

use dashmap::DashMap;

use crate::backend::remote::RemoteKernel;

/// State for the running Tauri application.
#[derive(Default)]
pub struct State {
    /// Current kernels running in the application.
    pub kernels: DashMap<String, RemoteKernel>,
}

impl State {
    /// Create a new state object.
    pub fn new() -> Self {
        Self::default()
    }
}
