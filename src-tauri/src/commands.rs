use std::path::PathBuf;
use std::{collections::HashMap, fmt::Display};

use serde_json::Value;

use crate::{
    app_handle,
    savesync::{
        config_paths::get_pluginfiles,
        plugin::{load_plugin, Plugin},
    },
};

#[tauri::command]
pub fn get_plugins() -> Vec<HashMap<String, Value>> {
    load_plugins()
        .into_iter()
        .filter_map(|(path, plugin)| {
            plugin.info().map_or_else(
                |e| {
                    emit_error(format!(
                        "Failed to run Info() in {}: {e}",
                        path.file_name().unwrap().to_string_lossy()
                    ));
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

fn load_plugins() -> HashMap<PathBuf, Plugin> {
    get_pluginfiles()
        .into_iter()
        .filter_map(|path| {
            load_plugin(&path).map_or_else(
                |e| {
                    emit_error(e.to_string());
                    None
                },
                |x| Some((path, x)),
            )
        })
        .collect()
}
