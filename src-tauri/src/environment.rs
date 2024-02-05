//! Metadata about the kernel environment and file system configuration.

use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
};

use futures_util::future::join_all;
use serde::Deserialize;
use tokio::fs;

/// Information parsed from the `kernel.json` file.
///
/// See <https://jupyter-client.readthedocs.io/en/latest/kernels.html#kernel-specs>
/// for more information about the kernel spec format.
#[derive(Deserialize, Debug)]
pub struct KernelSpec {
    /// List of command-line arguments to start the kernel.
    pub argv: Vec<String>,

    /// The display name of the kernel.
    pub display_name: String,

    /// The language of the kernel.
    pub language: String,

    /// The interrupt mode of the kernel ("signal" by default).
    #[serde(default)]
    pub interrupt_mode: KernelInterruptMode,

    /// A dictionary of environment variables to set for the kernel.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

/// The interrupt mode of the kernel.
#[derive(Default, Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KernelInterruptMode {
    /// Interrupts are communicated by sending a signal.
    #[default]
    Signal,

    /// Interrupts are communicated by messages on the control socket.
    Message,
}

/// Lists the ordered search path to find installable data files.
///
/// This is specified in
/// <https://docs.jupyter.org/en/latest/use/jupyter-directories.html#data-files>.
async fn data_search_paths(interpreter_prefix: Option<&str>) -> Vec<String> {
    let mut dirs = Vec::new();
    if let Ok(jupyter_path) = env::var("JUPYTER_PATH") {
        let pathsep = if cfg!(windows) { ";" } else { ":" };
        dirs.extend(jupyter_path.split(pathsep).map(String::from));
    }
    if let Ok(jupyter_data_dir) = env::var("JUPYTER_DATA_DIR") {
        dirs.push(jupyter_data_dir);
    } else {
        #[cfg(windows)]
        dirs.push(env::var("AppData").unwrap() + "\\jupyter");
        #[cfg(target_os = "macos")]
        dirs.push(env::var("HOME").unwrap() + "/Library/Jupyter");
        #[cfg(target_os = "linux")]
        match env::var("XDG_DATA_HOME") {
            Ok(xdg_data_home) => dirs.push(xdg_data_home + "/jupyter"),
            Err(_) => dirs.push(env::var("HOME").unwrap() + "/.local/share/jupyter"),
        }
    }
    if let Some(prefix) = interpreter_prefix {
        dirs.push(prefix.to_string() + "/share/jupyter");
    }
    #[cfg(windows)]
    dirs.push(env::var("ProgramData").unwrap() + "\\jupyter");
    #[cfg(unix)]
    dirs.extend([
        String::from("/usr/share/jupyter"),
        String::from("/usr/local/share/jupyter"),
    ]);
    dirs
}

/// List all available kernels from the environment, checking the search path.
pub async fn list_kernels(interpreter_prefix: Option<&str>) -> Vec<(PathBuf, KernelSpec)> {
    let dirs = data_search_paths(interpreter_prefix).await;
    join_all(dirs.iter().map(|path| list_kernels_from_path(path)))
        .await
        .into_iter()
        .flatten()
        .collect()
}

/// List all the available kernels from a given path.
async fn list_kernels_from_path(path: &str) -> Vec<(PathBuf, KernelSpec)> {
    let mut kernels = Vec::new();
    let Ok(mut items) = fs::read_dir(Path::new(path).join("kernels")).await else {
        return kernels;
    };
    while let Ok(Some(entry)) = items.next_entry().await {
        let kernel_path = entry.path().join("kernel.json");
        if let Ok(kernel_json) = fs::read(&kernel_path).await {
            if let Ok(kernel) = serde_json::from_slice(&kernel_json) {
                kernels.push((kernel_path, kernel));
            }
        }
    }
    kernels
}

/// Get the configured directory for data files.
pub fn data_dir() -> String {
    if let Ok(jupyter_data_dir) = env::var("JUPYTER_DATA_DIR") {
        return jupyter_data_dir;
    }

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            env::var("AppData").unwrap() + "\\jupyter"
        } else if #[cfg(target_os = "macos")] {
            env::var("HOME").unwrap() + "/Library/Jupyter"
        } else if #[cfg(target_os = "linux")] {
            match env::var("XDG_DATA_HOME") {
                Ok(xdg_data_home) => xdg_data_home + "/jupyter",
                Err(_) => env::var("HOME").unwrap() + "/.local/share/jupyter",
            }
        } else {
            panic!("Unsupported platform, cannot determine data directory")
        }
    }
}

/// Get the configured directory where runtime connection files are stored.
pub fn runtime_dir() -> String {
    match env::var("JUPYTER_RUNTIME_DIR") {
        Ok(jupyter_runtime_dir) => jupyter_runtime_dir,
        Err(_) => {
            let d = data_dir();
            if cfg!(windows) {
                d + "\\runtime"
            } else {
                d + "/runtime"
            }
        }
    }
}
