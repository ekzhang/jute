//! Shared code to open windows in Jute and notebooks.

use std::path::Path;

use anyhow::Context;
use tauri::{AppHandle, Manager, Runtime, WebviewWindow, WebviewWindowBuilder};
use uuid::Uuid;

/// Initializes window size, min width, and other common settings on the
/// builder.
pub fn initialize_builder<'a, R: Runtime, M: Manager<R>>(
    manager: &'a M,
    path: &str,
) -> WebviewWindowBuilder<'a, R, M> {
    // Generate a unique window label since duplicates are not allowed.
    let label = format!("jute-window-{}", Uuid::new_v4());

    let url = tauri::WebviewUrl::App(path.trim_start_matches('/').into());

    #[allow(unused_mut)]
    let mut builder = WebviewWindowBuilder::new(manager, &label, url)
        .title("Jute")
        .inner_size(960.0, 800.0)
        .min_inner_size(720.0, 600.0)
        .fullscreen(false)
        .resizable(true);

    #[cfg(target_os = "macos")]
    {
        // These methods are only available on macOS.
        builder = builder.title_bar_style(tauri::TitleBarStyle::Overlay);
        builder = builder.hidden_title(true);
    }

    builder
}

/// Opens a window with the home page.
pub fn open_home<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<WebviewWindow<R>> {
    initialize_builder(app, "/").build()
}

/// Opens a window with the notebook file at the given path.
pub fn open_notebook_path<R: Runtime>(
    app: &AppHandle<R>,
    file: &Path,
) -> tauri::Result<WebviewWindow<R>> {
    let query = serde_urlencoded::to_string([("path", file.to_string_lossy())])
        .context("could not encode path")?;
    initialize_builder(app, &format!("/notebook?{query}")).build()
}
