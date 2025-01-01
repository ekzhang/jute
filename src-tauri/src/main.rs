// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

use jute::{
    backend::{
        commands::{self, RunCellEvent},
        local::{environment, LocalKernel},
    },
    state::State,
    Error,
};
use sysinfo::System;
use tauri::{ipc::Channel, LogicalSize, Manager};
use tracing::info;

#[tauri::command]
async fn cpu_usage() -> f32 {
    let mut system = System::new();
    system.refresh_cpu();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    system.refresh_cpu();
    system.global_cpu_info().cpu_usage()
}

/// Start a new Jupyter kernel.
#[tauri::command]
async fn start_kernel(spec_name: &str, state: tauri::State<'_, State>) -> Result<String, Error> {
    // TODO: Save the client in a better place.
    // let client = JupyterClient::new("", "")?;

    // Temporary hack to just start a kernel locally with ZeroMQ.
    let kernels = environment::list_kernels(None).await;
    let mut kernel_spec = match kernels
        .iter()
        .find(|(path, _spec)| path.file_name().and_then(|s| s.to_str()) == Some(spec_name))
    {
        Some((_, kernel_spec)) => kernel_spec.clone(),
        None => {
            return Err(Error::KernelConnect(format!(
                "no kernel named {spec_name:?} found"
            )))
        }
    };

    if kernel_spec.argv[0] == "python" {
        if let Ok(python_path) = env::var("PYTHON_PATH") {
            kernel_spec.argv[0] = python_path;
        } else {
            // Temporary hack
            kernel_spec.argv[0] = "/opt/homebrew/bin/python3.11".into();
        }
    }

    let kernel = LocalKernel::start(&kernel_spec).await?;

    let info = commands::kernel_info(kernel.conn()).await?;
    info!(banner = info.banner, "started new jute kernel");

    let kernel_id = String::from(kernel.id());
    state.kernels.insert(kernel_id.clone(), kernel);
    Ok(kernel_id)
}

#[tauri::command]
async fn stop_kernel(kernel_id: &str, state: tauri::State<'_, State>) -> Result<(), Error> {
    info!("stopping jute kernel {kernel_id}");
    let (_, mut kernel) = state
        .kernels
        .remove(kernel_id)
        .ok_or(Error::KernelDisconnect)?;
    kernel.kill().await?;
    Ok(())
}

#[tauri::command]
async fn run_cell(
    kernel_id: &str,
    code: &str,
    on_event: Channel<RunCellEvent>,
    state: tauri::State<'_, State>,
) -> Result<(), Error> {
    let conn = state
        .kernels
        .get(kernel_id)
        .ok_or(Error::KernelDisconnect)?
        .conn()
        .clone();

    let rx = commands::run_cell(&conn, code).await?;
    while let Ok(event) = rx.recv().await {
        if on_event.send(event).is_err() {
            break;
        }
    }
    Ok(())
}

fn main() {
    tracing_subscriber::fmt().init();

    #[allow(unused_mut)]
    let mut app = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        app = app.plugin(jute::plugins::macos_traffic_lights::init());
    }

    app.manage(State::new())
        .invoke_handler(tauri::generate_handler![
            cpu_usage,
            start_kernel,
            stop_kernel,
            run_cell,
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
