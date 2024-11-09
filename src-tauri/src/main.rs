// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Stdio;

use jute::{
    jupyter_client::{JupyterClient, Kernel},
    state::State,
    Error,
};
use serde::Serialize;
use sysinfo::System;
use tauri::{
    ipc::{Channel, IpcResponse},
    LogicalSize, Manager,
};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

#[tauri::command]
async fn cpu_usage() -> f32 {
    let mut system = System::new();
    system.refresh_cpu();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    system.refresh_cpu();
    system.global_cpu_info().cpu_usage()
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum RunPythonEvent {
    Stdout(String),
    Stderr(String),
    Done { status: i32 },
}

async fn stream_to_ipc<T: IpcResponse + Clone>(
    mut stream: impl AsyncRead + Unpin,
    channel: Channel<T>,
    make_event: impl Fn(String) -> T,
) {
    let mut buf = [0; 8192];
    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }
        let s = String::from_utf8_lossy(&buf[..n]);
        if channel.send(make_event(s.into())).is_err() {
            break;
        }
    }
}

/// Run some Python code. This is a temporary placeholder for the future, real
/// implementation based on interactive kernels.
#[tauri::command]
async fn run_python(source_code: &str, on_event: Channel<RunPythonEvent>) -> Result<(), Error> {
    let mut child = Command::new("python3")
        .args(["-u", "-c", source_code])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Error::Subprocess)?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let h1 = stream_to_ipc(stdout, on_event.clone(), RunPythonEvent::Stdout);
    let h2 = stream_to_ipc(stderr, on_event.clone(), RunPythonEvent::Stderr);

    let (_, _, result) = tokio::join!(h1, h2, child.wait());

    let status = result.map_err(Error::Subprocess)?;

    // On Unix, the code is None if the process was killed by a signal.
    let code = status.code().unwrap_or(-1);
    _ = on_event.send(RunPythonEvent::Done { status: code });

    Ok(())
}

/// Start a new Jupyter kernel.
#[tauri::command]
async fn start_kernel(spec_name: &str, state: tauri::State<'_, State>) -> Result<String, Error> {
    // TODO: Save the client in a better place.
    let client = JupyterClient::new("", "")?;
    let kernel = Kernel::start(&client, spec_name).await?;
    let id = String::from(kernel.id());
    state.kernels.insert(id.clone(), kernel);
    Ok(id)
}

fn main() {
    tracing_subscriber::fmt().init();

    tauri::Builder::default()
        .manage(State::new())
        .invoke_handler(tauri::generate_handler![
            cpu_usage,
            run_python,
            start_kernel,
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.set_min_size(Some(LogicalSize::new(720.0, 600.0)))?;
            // main_window.set_size(LogicalSize::new(720.0, 800.0))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
