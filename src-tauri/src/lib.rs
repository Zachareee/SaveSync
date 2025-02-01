// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod savesync;

use commands::{emit_listeners, get_plugins, load_plugins};
use savesync::plugin::Plugin;
use std::{
    collections::HashMap,
    ffi::OsString,
    sync::{Arc, Mutex, OnceLock},
};
use tauri::{AppHandle, Manager};

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();

#[derive(Default)]
struct AppState {
    plugins: HashMap<Arc<OsString>, Plugin>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_plugins])
        .setup(|app| {
            emit_listeners(app);
            app.manage(Mutex::new(AppState {
                plugins: load_plugins(),
            }));

            APP_INSTANCE.set(app.app_handle().to_owned()).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn app_handle() -> AppHandle {
    APP_INSTANCE.get().unwrap().to_owned()
}
