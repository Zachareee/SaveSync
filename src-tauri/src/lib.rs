// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod savesync;

use commands::{emit_listeners, get_plugins};
use savesync::config_paths::config;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};
use tauri::{AppHandle, Emitter, Manager, State};

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();

pub struct AppState {
    plugin: PathBuf,
    path_mapping: HashMap<String, PathBuf>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_plugins])
        .setup(|app| {
            emit_listeners(app);

            if let Ok(plugin) = fs::read_to_string(config().join("last_plugin.txt")) {
                app.manage(Mutex::new(AppState {
                    plugin: plugin.into(),
                    path_mapping: HashMap::new(),
                }));
            }

            APP_INSTANCE.set(app.app_handle().to_owned()).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn app_handle() -> AppHandle {
    APP_INSTANCE.get().unwrap().to_owned()
}

pub fn app_state<'a>(handle: &'a AppHandle) -> State<'a, Mutex<AppState>> {
    handle.state::<Mutex<AppState>>()
}

pub fn app_emit<S>(event: &str, payload: S)
where
    S: Serialize + Clone,
{
    app_handle()
        .emit(event, payload)
        .expect("Unable to emit event")
}
