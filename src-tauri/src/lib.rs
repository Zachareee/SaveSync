// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod savesync;

use commands::{emit_listeners, get_fmap, get_plugins, saved_plugin};
use savesync::state::{read_state, save_state, AppState};
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State};

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_plugins,
            get_fmap,
            saved_plugin
        ])
        .setup(|app| {
            emit_listeners(app);

            app.manage(Mutex::new(read_state()));

            APP_INSTANCE.set(app.app_handle().to_owned()).unwrap();
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Error while building tauri application")
        .run(|_handle, event| match event {
            RunEvent::ExitRequested { .. } => {
                save_state();
            }
            _ => (),
        })
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
