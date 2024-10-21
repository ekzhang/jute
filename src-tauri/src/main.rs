// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io;

use jute::{
    jupyter_client::{JupyterClient, Kernel},
    state::State,
    Error,
};
use sysinfo::System;
use tauri::{LogicalSize, Manager};
use tokio::process::Command;

#[tauri::command]
async fn cpu_usage() -> f32 {
    let mut system = System::new();
    system.refresh_cpu();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    system.refresh_cpu();
    system.global_cpu_info().cpu_usage()
}

/// Run some Python code. This is a temporary placeholder for the future, real
/// implementation based on interactive kernels.
#[tauri::command]
async fn run_python(source_code: &str) -> Result<String, Error> {
    let output = Command::new("python3")
        .args(["-c", source_code])
        .output()
        .await
        .map_err(Error::Subprocess)?;
    if !output.status.success() {
        return Err(Error::Subprocess(io::Error::from_raw_os_error(
            output.status.code().unwrap(),
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
