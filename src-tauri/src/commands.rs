use serde_json::from_str;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fmt::Display};

use tauri::{Emitter, Event, Listener, Manager};

use crate::{
    app_handle,
    savesync::{
        config_paths::get_pluginfiles,
        plugin::{load_plugin, Plugin, PluginInfo},
    },
    AppState,
};

pub fn emit_listeners(app: &tauri::App) {
    [("init", init_listener)]
        .into_iter()
        .for_each(|(event, handler)| {
            app.listen(event, handler);
        });
}

fn init_listener(event: Event) {
    if let Some(x) = load_plugins().get(&from_str::<OsString>(event.payload()).unwrap()) {
        app_handle()
            .emit("init_result", x.init().map_err(|e| emit_error(e)).is_ok())
            .expect("Unable to emit event");
    }
}

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    app_handle()
        .state::<Mutex<AppState>>()
        .lock()
        .expect("Unable to obtain lock to retrieve app state")
        .plugins
        .iter()
        .filter_map(|(path, plugin)| {
            plugin.info().map_or_else(
                |e| {
                    emit_error(format!("Failed to run Info() in {:?}: {e}", path));
                    None
                },
                Some,
            )
        })
        .collect()
}

#[tauri::command]
pub fn emit_error<T>(e: T)
where
    T: Display + Sync + Send + 'static,
{
    std::thread::spawn(move || {
        let _ = tauri::WebviewWindowBuilder::new(
            &app_handle(),
            "error",
            tauri::WebviewUrl::App(format!("error.html?msg={e}").into()),
        )
        .title("Error")
        .build();
    });
}

pub fn load_plugins() -> HashMap<Arc<OsString>, Plugin> {
    get_pluginfiles()
        .into_iter()
        .filter_map(|path| {
            load_plugin(&path).map_or_else(
                |e| {
                    emit_error(e.to_string());
                    None
                },
                |x| Some((x.filename(), x)),
            )
        })
        .collect()
}
