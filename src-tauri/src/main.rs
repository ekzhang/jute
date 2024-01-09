// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sysinfo::System;
use tauri::{LogicalSize, Manager};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn cpu_usage() -> f32 {
    let mut system = System::new();
    system.refresh_cpu();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    system.refresh_cpu();
    system.global_cpu_info().cpu_usage()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, cpu_usage])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            main_window.set_min_size(Some(LogicalSize::new(600.0, 400.0)))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
