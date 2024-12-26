//! Code that starts local kernels to be `jupyter-server` compatible.
//!
//! This is currently unused while Jute relies on `jupyter-server`, but in the
//! future it could replace the Jupyter installation by directly invoking
//! kernels, or introduce new APIs for developer experience.

use std::process::Stdio;

use serde_json::json;
use tokio::fs;
use tokio::net::TcpListener;
use uuid::Uuid;

use self::environment::KernelSpec;
use super::{create_zeromq_connection, KernelConnection};
use crate::Error;

pub mod environment;

/// Represents a connection to an active kernel.
pub struct LocalKernel {
    child: tokio::process::Child,
    kernel_id: String,

    spec: KernelSpec,
    conn: KernelConnection,
}

impl LocalKernel {
    /// Start a new kernel based on a spec, and connect to it.
    pub async fn start(spec: &KernelSpec) -> Result<Self, Error> {
        let (control_port, shell_port, iopub_port, stdin_port, heartbeat_port) = tokio::try_join!(
            get_available_port(),
            get_available_port(),
            get_available_port(),
            get_available_port(),
            get_available_port(),
        )?;
        let signing_key = Uuid::new_v4().to_string();
        let connection_file = json!({
            "control_port": control_port,
            "shell_port": shell_port,
            "iopub_port": iopub_port,
            "stdin_port": stdin_port,
            "hb_port": heartbeat_port,
            "transport": "tcp",
            "ip": "127.0.0.1",
            "signature_scheme": "hmac-sha256",
            "key": signing_key,
        });

        let kernel_id = Uuid::new_v4().to_string();
        let runtime_dir = environment::runtime_dir();
        let connection_filename = runtime_dir + &format!("jute-{kernel_id}.json");
        fs::write(&connection_filename, connection_file.to_string())
            .await
            .map_err(|err| {
                Error::KernelConnect(format!("could not write connection file: {err}"))
            })?;

        if spec.argv.is_empty() {
            return Err(Error::KernelConnect("kernel spec has no argv".into()));
        }
        let argv: Vec<String> = spec
            .argv
            .iter()
            .map(|s| s.replace("{connection_file}", &connection_filename))
            .collect();
        // TODO: Handle spec.env
        let child = tokio::process::Command::new(&argv[0])
            .args(&argv[1..])
            .kill_on_drop(true)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(Error::Subprocess)?;

        let conn = create_zeromq_connection(
            shell_port,
            control_port,
            iopub_port,
            stdin_port,
            heartbeat_port,
            &signing_key,
        )
        .await?;

        Ok(Self {
            child,
            kernel_id,
            spec: spec.clone(),
            conn,
        })
    }

    /// Get the kernel ID.
    pub fn id(&self) -> &str {
        &self.kernel_id
    }

    /// Get the kernel connection object.
    pub fn conn(&self) -> &KernelConnection {
        &self.conn
    }

    /// Return the spec used to start the kernel.
    pub fn spec(&self) -> &KernelSpec {
        &self.spec
    }

    /// Check if the kernel is still alive.
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Kill the kernel by sending a SIGKILL signal.
    pub async fn kill(&mut self) -> Result<(), Error> {
        self.child.kill().await.map_err(Error::Subprocess)
    }
}

async fn get_available_port() -> Result<u16, Error> {
    let addr = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| Error::KernelConnect(format!("could not get available port: {err}")))?
        .local_addr()
        .map_err(|_| Error::KernelConnect("tcp listener has no local address".into()))?;
    Ok(addr.port())
}
