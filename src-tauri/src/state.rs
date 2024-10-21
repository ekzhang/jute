//! Defines state and stores for the Tauri application.

use dashmap::DashMap;

use crate::jupyter_client::Kernel;

/// State for the running Tauri application.
#[derive(Default)]
pub struct State {
    /// Current kernels running in the application.
    pub kernels: DashMap<String, Kernel>,
}

impl State {
    /// Create a new state object.
    pub fn new() -> Self {
        Self::default()
    }
}
