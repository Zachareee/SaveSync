// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod listeners;
mod savesync;

use commands::{get_envpaths, get_filetree, get_mapping, get_plugins, set_mapping};
use listeners::emit_listeners;
use savesync::store::AppStore;
use serde::Serialize;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter, Manager, RunEvent};

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();
static APP_STORE: OnceLock<Arc<AppStore>> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_plugins,
            get_filetree,
            get_mapping,
            set_mapping,
            get_envpaths,
        ])
        .setup(|app| {
            emit_listeners(app);

            let _ = APP_STORE.set(Arc::new(AppStore::new(app)));

            app.manage(Mutex::new(Vec::<String>::new()));

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

pub fn app_emit<S>(event: &str, payload: S)
where
    S: Serialize + Clone,
{
    app_handle()
        .emit(event, payload)
        .expect("Unable to emit event")
}
