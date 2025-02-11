// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod savesync;

use commands::{emit_listeners, get_fmap, get_plugins};
use notify_debouncer_full::{notify::RecommendedWatcher, Debouncer, RecommendedCache};
use savesync::config_paths::{config, get_tag_paths};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};
use tauri::{AppHandle, Emitter, Manager, State};

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();

#[allow(dead_code)]
pub struct AppState {
    plugin: PathBuf,
    path_mapping: HashMap<String, PathBuf>,
    watchers: HashMap<PathBuf, Debouncer<RecommendedWatcher, RecommendedCache>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_plugins, get_fmap])
        .setup(|app| {
            emit_listeners(app);

            app.manage(Mutex::new(AppState {
                plugin: fs::read_to_string(config().join("last_plugin.txt"))
                    .unwrap_or_default()
                    .into(),
                path_mapping: get_tag_paths().unwrap_or_default(),
                watchers: Default::default(),
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
