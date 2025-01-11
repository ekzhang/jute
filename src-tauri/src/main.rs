// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, io, path::PathBuf};

use jute::{
    backend::{
        commands::{self, RunCellEvent},
        local::{environment, LocalKernel},
        notebook::NotebookRoot,
    },
    entity::{Entity, EntityId},
    state::State,
    Error,
};
use sysinfo::System;
#[allow(unused_imports)]
use tauri::Manager;
use tauri::{ipc::Channel, AppHandle};
use tauri_plugin_shell::ShellExt;
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
async fn get_notebook(path: &str) -> Result<NotebookRoot, Error> {
    info!("getting notebook at {path}");

    let contents = tokio::fs::read_to_string(path)
        .await
        .map_err(Error::Filesystem)?;
    Ok(serde_json::from_str(&contents)?)
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

#[tauri::command]
async fn venv_create(python_version: &str, app: AppHandle) -> Result<EntityId, Error> {
    let venv_id = EntityId::new(Entity::Venv);
    let venv_path = app
        .path()
        .app_data_dir()?
        .join("venv")
        .join(venv_id.to_string());

    let output = app
        .shell()
        .sidecar("uv")?
        .args(["venv", "--no-project", "--seed", "--relocatable"])
        .args([
            "--python",
            python_version,
            "--python-preference",
            "only-managed",
        ])
        .arg(&venv_path)
        .output()
        .await?;

    if output.status.success() {
        info!("created venv at {venv_path:?}");
        Ok(venv_id)
    } else {
        let message = String::from_utf8_lossy(&output.stderr);
        Err(Error::Subprocess(io::Error::new(
            io::ErrorKind::Other,
            message.trim(),
        )))
    }
}

#[tauri::command]
async fn venv_list(app: AppHandle) -> Result<Vec<EntityId>, Error> {
    let venv_dir = app.path().app_data_dir()?.join("venv");
    let mut venvs = Vec::new();
    let mut it = tokio::fs::read_dir(venv_dir)
        .await
        .map_err(Error::Filesystem)?;
    while let Some(entry) = it.next_entry().await.map_err(Error::Filesystem)? {
        if entry.file_type().await.is_ok_and(|f| f.is_dir()) {
            if let Ok(venv_id) = entry.file_name().into_string() {
                if let Ok(venv_id) = venv_id.parse::<EntityId>() {
                    venvs.push(venv_id);
                }
            }
        }
    }
    Ok(venvs)
}

#[tauri::command]
async fn venv_delete(venv_id: EntityId, app: AppHandle) -> Result<bool, Error> {
    let venv_dir = app.path().app_data_dir()?.join("venv");
    let venv_path = venv_dir.join(venv_id.to_string());
    if tokio::fs::metadata(&venv_path).await.is_ok() {
        tokio::fs::remove_dir_all(&venv_path)
            .await
            .map_err(Error::Filesystem)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Handle file associations opened in the application.
///
/// Jute registers itself as an application to open `.ipynb` files, which are
/// the file type for Jupyter Notebooks. This function is called when the user
/// double-clicks on a notebook file to open it with Jute.
///
/// Depending on the operating system, it will either launch a new process with
/// the file in `argv[1]` or send a [`tauri::RunEvent::Opened`] event. There may
/// be multiple file paths in `argv`, and they can be provided either as paths
/// or in the `file://` URL format.
///
/// Currently, each file should be opened as a separate window.
///
/// This function's code is adapted from the [`file-associations`] example in
/// the Tauri docs.
///
/// [`file-associations`]: https://github.com/tauri-apps/tauri/blob/tauri-v2.2.0/examples/file-associations/src-tauri/src/main.rs
fn handle_file_associations(
    app: &AppHandle,
    files: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    for file in files {
        jute::window::open_notebook_path(app, file)?;
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            cpu_usage,
            start_kernel,
            stop_kernel,
            run_cell,
            get_notebook,
            venv_create,
            venv_list,
            venv_delete,
        ])
        .setup(|app| {
            // Parse files that were opened via CLI arguments (Windows + Linux).
            if cfg!(any(windows, target_os = "linux")) {
                let mut files = Vec::new();

                for maybe_file in env::args().skip(1) {
                    // Skip flags like -f or --flag
                    if maybe_file.starts_with('-') {
                        continue;
                    }
                    // Handle `file://` path URLs and skip other URLs.
                    if let Ok(url) = url::Url::parse(&maybe_file) {
                        if url.scheme() == "file" {
                            if let Ok(path) = url.to_file_path() {
                                files.push(path);
                            }
                        }
                    } else {
                        files.push(PathBuf::from(maybe_file));
                    }
                }

                if files.is_empty() {
                    // Open a default window if no files were provided (this is if you opened the
                    // app in the launcher, for instance).
                    jute::window::open_home(app.handle())?;
                } else {
                    handle_file_associations(app.handle(), &files)?;
                }
            }

            Ok(())
        })
        .menu(jute::menu::setup_menu)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(
            #[allow(unused_variables)]
            |app, event| {
                // Handle files opened in macOS.
                #[cfg(target_os = "macos")]
                match event {
                    tauri::RunEvent::Opened { urls } => {
                        let files = urls
                            .into_iter()
                            .filter_map(|url| url.to_file_path().ok())
                            .collect::<Vec<_>>();
                        handle_file_associations(app, &files).unwrap();
                    }
                    tauri::RunEvent::Ready => {
                        // If no files were opened, open a default window.
                        if app.webview_windows().is_empty() {
                            jute::window::open_home(app).unwrap();
                        }
                    }
                    _ => {}
                }
            },
        );
}
