use std::ffi::OsString;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};

use serde_json::from_str;
use tauri::{Emitter, Event, Listener};

use crate::{
    app_handle,
    savesync::{
        config_paths::get_pluginfiles,
        plugin::{load_plugin, Plugin, PluginInfo},
    },
};

pub fn emit_listeners(app: &tauri::App) {
    let arr: Vec<(&str, fn(Event))> = vec![("init", init_listener)];
    arr.into_iter().for_each(|(event, handler)| {
        app.listen(event, handler);
    });
}

// async to prevent UI thread from freezing
fn init_listener(event: Event) {
    let path: OsString = from_str::<OsString>(event.payload()).unwrap();

    match load_plugins().get(&path) {
        Some(plugin) => {
            app_handle()
                .emit(
                    "init_result",
                    plugin.init().map_err(|e| emit_error(e)).is_ok(),
                )
                .expect("Unable to emit event");
        }
        None => {
            emit_error(format!("{path:?} not found"));
        }
    }
}

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    load_plugins()
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
