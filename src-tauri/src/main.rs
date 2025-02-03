// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, path::PathBuf};

use jute::state::State;
use tauri::AppHandle;
#[allow(unused_imports)]
use tauri::Manager;

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
            jute::commands::kernel_usage_info,
            jute::commands::start_kernel,
            jute::commands::stop_kernel,
            jute::commands::run_cell,
            jute::commands::get_notebook,
            jute::commands::venv::venv_list_python_versions,
            jute::commands::venv::venv_create,
            jute::commands::venv::venv_list,
            jute::commands::venv::venv_delete,
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
