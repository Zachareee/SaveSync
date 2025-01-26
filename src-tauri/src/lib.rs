// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod savesync;

use commands::get_plugins;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_plugins])
        .setup(|app| {
            APP_INSTANCE.set(app.app_handle().to_owned()).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn app_handle() -> AppHandle {
    APP_INSTANCE.get().unwrap().to_owned()
}
