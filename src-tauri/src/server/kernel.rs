//! Speak the kernel protocol directly to a running subprocess.

#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use bytes::Bytes;
use serde_json::json;
use tokio::net::TcpListener;
use uuid::Uuid;
use zeromq::Socket;

use super::environment::{self, KernelSpec};

/// Represents a connection to an active kernel.
pub struct Kernel {
    child: tokio::process::Child,

    spec: KernelSpec,
    signing_key: String,

    control: zeromq::DealerSocket,
    shell: zeromq::DealerSocket,
    iopub: zeromq::SubSocket,
    stdin: zeromq::DealerSocket,
    heartbeat: zeromq::ReqSocket,
}

impl Kernel {
    /// Start a new kernel based on a spec, and connect to it.
    pub async fn start(spec: &KernelSpec) -> Result<Self> {
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

        let runtime_dir = environment::runtime_dir();
        let connection_filename = runtime_dir + &format!("jute-{}.json", Uuid::new_v4());
        std::fs::write(&connection_filename, connection_file.to_string())
            .context("could not write connection file")?;

        if spec.argv.is_empty() {
            bail!("kernel spec has no argv");
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
            .spawn()?;

        let mut control = zeromq::DealerSocket::new();
        control
            .connect(&format!("tcp://127.0.0.1:{control_port}"))
            .await?;
        let mut shell = zeromq::DealerSocket::new();
        shell
            .connect(&format!("tcp://127.0.0.1:{shell_port}"))
            .await?;
        let mut iopub = zeromq::SubSocket::new();
        iopub
            .connect(&format!("tcp://127.0.0.1:{iopub_port}"))
            .await?;
        iopub.subscribe("").await?;
        let mut stdin = zeromq::DealerSocket::new();
        stdin
            .connect(&format!("tcp://127.0.0.1:{stdin_port}"))
            .await?;
        let mut heartbeat = zeromq::ReqSocket::new();
        heartbeat
            .connect(&format!("tcp://127.0.0.1:{heartbeat_port}"))
            .await?;

        Ok(Self {
            child,
            spec: spec.clone(),
            signing_key,
            control,
            shell,
            iopub,
            stdin,
            heartbeat,
        })
    }
}

async fn get_available_port() -> Result<u16> {
    let addr = TcpListener::bind("127.0.0.1:0")
        .await
        .context("could not get available port")?
        .local_addr()
        .context("tcp listener has no local address")?;
    Ok(addr.port())
}

/// Sign a message using HMAC-SHA256 with the kernel's signing key.
fn sign_message(digest_key: &str, bytes: &[Bytes]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(digest_key.as_bytes()).unwrap();
    for b in bytes {
        mac.update(b);
    }
    format!("{:x}", mac.finalize().into_bytes())
}
