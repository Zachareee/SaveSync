// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod listeners;
mod savesync;

use commands::{get_envpaths, get_mapping, get_plugins, get_watched_folders, set_mapping};
use listeners::emit_listeners;
use savesync::store::AppStore;
use std::{
    collections::HashMap,
    ffi::OsString,
    sync::{Arc, Mutex, OnceLock},
};
use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_deep_link::DeepLinkExt;

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();
static APP_STORE: OnceLock<Arc<AppStore>> = OnceLock::new();
const REDIRECT_URL: &str = "savesync://tokens";

struct AppState {
    pub tags: Vec<String>,
    pub buffers: HashMap<(String, OsString), Vec<u8>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            tags: Vec::new(),
            buffers: HashMap::new(),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_, _, _| {}));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![
            get_plugins,
            get_mapping,
            set_mapping,
            get_envpaths,
            get_watched_folders,
        ])
        .setup(|app| {
            emit_listeners(app);

            let _ = APP_STORE.set(Arc::new(AppStore::new(app)));

            app.manage(Mutex::new(AppState::default()));

            #[cfg(desktop)]
            app.deep_link().on_open_url(|e| {
                println!("Urls: {:?}", e.urls());
            });

            APP_INSTANCE.set(app.app_handle().to_owned()).unwrap();
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Error while building tauri application")
        .run(|_, event| match event {
            RunEvent::ExitRequested { .. } => {
                app_store().save().unwrap();
            }
            _ => (),
        })
}

pub fn app_handle() -> AppHandle {
    APP_INSTANCE.get().unwrap().to_owned()
}

pub fn app_store() -> Arc<AppStore> {
    APP_STORE.get().unwrap().clone()
}
