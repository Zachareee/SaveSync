use std::path::PathBuf;
use std::{collections::HashMap, fmt::Display};

//use serde::Serialize;
//use tauri::{AppHandle, Emitter};

use crate::savesync::{
    config_paths::get_pluginfiles,
    plugin::{load_plugin, Plugin, PluginInfo},
};

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    load_plugins()
        .into_iter()
        .filter_map(|(path, plugin)| {
            plugin.info().map_or_else(
                |_| {
                    emit_error(format!(
                        "Failed to run Info() in {}",
                        path.file_name().unwrap().to_string_lossy()
                    ))
                },
                Some,
            )
        })
        .collect()
}

//#[tauri::command]
//pub fn emit_error<T>(app: AppHandle, e: impl Serialize) -> Option<T> {
//    app.emit("error", e);
pub fn emit_error<T>(e: impl Display) -> Option<T> {
    println!("{e}");
    None
}

fn load_plugins() -> HashMap<PathBuf, Plugin> {
    get_pluginfiles()
        .into_iter()
        .filter_map(|path| load_plugin(&path).map_or_else(emit_error, |x| Some((path, x))))
        .collect()
}
