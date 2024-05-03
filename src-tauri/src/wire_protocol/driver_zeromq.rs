use super::KernelConnection;
use crate::Error;

/// Connect to Jupyter via ZeroMQ to a local kernel.
pub fn create_zeromq_connection() -> Result<KernelConnection, Error> {
    todo!("zeromq kernel connection")
}
