//! Sets up the native window menu with file, about, and other commands.
//!
//! Menus in Tauri v2 are attached to each window for Windows and Linux, but
//! they are global for macOS. Each item in the menu has a `MenuId` string
//! attached to it, and it emits a `MenuEvent` when clicked.
//!
//! There is no way to associate a `MenuEvent` with a specific window other than
//! creating separate menus for each window with a different UUID. This is
//! awkward, so we'll instead take the simpler approach of iterating through all
//! windows of the app and finding the focused one.
//!
//! Spacedrive has a good example of using the Menu API.
//! <https://github.com/spacedriveapp/spacedrive/blob/0.4.3/apps/desktop/src-tauri/src/menu.rs>

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{
        AboutMetadata, Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
        HELP_SUBMENU_ID, WINDOW_SUBMENU_ID,
    },
    AppHandle, Runtime,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_opener::OpenerExt;
use tracing::warn;
use ts_rs::TS;

/// The events that can be emitted as menu IDs.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    TS,
    strum::EnumString,
    strum::AsRefStr,
    strum::Display,
)]
pub enum MenuEvent {
    /// Open a notebook file.
    OpenFile,

    /// Open the issue tracker URL.
    ReportIssue,
}

/// Set up the menu for application windows.
///
/// This code was modified from the original source of [`Menu::default`],
/// customizing that menu to add new buttons.
pub fn setup_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    app.on_menu_event(move |app, event| {
        let Ok(event) = event.id().as_ref().parse::<MenuEvent>() else {
            warn!("unknown menu event: {:?}", event.id());
            return;
        };
        match event {
            MenuEvent::OpenFile => {
                let app = app.clone();
                app.dialog()
                    .file()
                    .add_filter("Jupyter Notebook", &["ipynb"])
                    .pick_file(move |path| {
                        if let Some(path) = path {
                            match path.into_path() {
                                Ok(path) => {
                                    _ = crate::window::open_notebook_path(&app, &path);
                                }
                                Err(err) => {
                                    app.dialog()
                                        .message(err.to_string())
                                        .kind(MessageDialogKind::Error)
                                        .show(|_| {});
                                }
                            }
                        }
                    });
            }
            MenuEvent::ReportIssue => {
                _ = app
                    .opener()
                    .open_url("https://github.com/ekzhang/jute/issues", None::<&str>);
            }
        }
    });

    let pkg_info = app.package_info();
    let config = app.config();
    let about_metadata = AboutMetadata {
        name: Some(pkg_info.name.clone()),
        version: Some(pkg_info.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config.bundle.publisher.clone().map(|p| vec![p]),
        icon: app.default_window_icon().cloned(),
        website: Some("https://github.com/ekzhang/jute".into()),
        website_label: Some("github.com/ekzhang/jute".into()),
        ..Default::default()
    };

    let mut menu = MenuBuilder::new(app);

    // App name submenu, only for macOS ("Jute").
    #[cfg(target_os = "macos")]
    {
        let app_menu = SubmenuBuilder::new(app, pkg_info.name.clone())
            .about(Some(about_metadata))
            .separator()
            .services()
            .separator()
            .hide()
            .hide_others()
            .separator()
            .quit()
            .build()?;
        menu = menu.item(&app_menu);
    }

    // File submenu.
    let file_menu = SubmenuBuilder::new(app, "File")
        .item(
            &MenuItemBuilder::with_id(MenuEvent::OpenFile, "Open File…")
                .accelerator("CmdOrCtrl+O")
                .build(app)?,
        )
        .items(&[
            // From the default menu: seems like this is not supported on Linux.
            #[cfg(not(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )))]
            &PredefinedMenuItem::close_window(app, None)?,
            // This is already in a different menu for macOS.
            #[cfg(not(target_os = "macos"))]
            &PredefinedMenuItem::quit(app, None)?,
        ])
        .build()?;

    // Edit submenu.
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .items(&[
            #[cfg(target_os = "macos")]
            &PredefinedMenuItem::fullscreen(app, None)?,
        ])
        .build()?;

    let window_menu = SubmenuBuilder::with_id(app, WINDOW_SUBMENU_ID, "Window")
        .minimize()
        .maximize()
        .separator()
        .close_window()
        .build()?;

    let help_menu = SubmenuBuilder::with_id(app, HELP_SUBMENU_ID, "Help")
        .items(&[
            #[cfg(not(target_os = "macos"))]
            &PredefinedMenuItem::about(app, None, Some(about_metadata))?,
        ])
        .text(MenuEvent::ReportIssue, "Report Issue")
        .build()?;

    let menu = menu
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

    Ok(menu)
}
