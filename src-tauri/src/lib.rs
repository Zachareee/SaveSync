// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod savesync;

use commands::{emit_listeners, get_plugins, load_plugins};
use savesync::{config_paths::config, plugin::Plugin};
use std::{
    fs,
    sync::{Arc, Mutex, OnceLock},
};
use tauri::{AppHandle, Manager, State};

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();

pub struct AppState {
    plugin: Plugin,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_plugins])
        .setup(|app| {
            emit_listeners(app);

            if let Some(plugin) = fs::read_to_string(config().join("last_plugin.txt"))
                .ok()
                .map_or(None, |plugin_name| {
                    load_plugins().remove(&Arc::new(plugin_name.into()))
                })
            {
                app.manage(Mutex::new(AppState { plugin }));
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
