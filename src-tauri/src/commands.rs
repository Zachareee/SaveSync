use std::collections::HashMap;
use std::ffi::OsString;
use std::{env, path};

use serde::{Deserialize, Serialize};

use crate::app_store;
use crate::listeners::required_tags;
use crate::savesync::watch::watched_folders;
use crate::savesync::{
    config_paths, emitter,
    plugin::{Plugin, PluginInfo},
    store::PathMapping,
};

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    config_paths::get_pluginfiles()
        .into_iter()
        .filter_map(|path| {
            Plugin::new(&path).map_or_else(
                |e| {
                    emitter::plugin_error(&path.to_string_lossy(), &e.to_string());
                    None
                },
                |x| {
                    x.info()
                        .map_err(|e| {
                            emitter::plugin_error(
                                &path.to_string_lossy(),
                                &format!("Failed to run Info() in {:?}: {e}", path),
                            )
                        })
                        .ok()
                },
            )
        })
        .collect()
}

#[derive(Deserialize, Serialize)]
pub struct Mappings {
    mapping: PathMapping,
    pub ignored: Vec<String>,
}

#[tauri::command]
pub fn get_mapping() -> Mappings {
    let mapping = app_store().path_mapping();
    let ignored = required_tags()
        .into_iter()
        .filter(|t| !&mapping.contains_key(t))
        .collect();
    Mappings { mapping, ignored }
}

#[tauri::command]
pub fn set_mapping(map: PathMapping) {
    app_store().set_mapping(map)
}

#[tauri::command]
pub fn get_envpaths() -> HashMap<String, OsString> {
    env::vars()
        .filter_map(|(k, v)| {
            path::absolute(&v)
                .ok()
                .filter(|p| p.exists())
                .map(|p| (k, p.into_os_string()))
        })
        .collect()
}

#[tauri::command]
pub fn get_watched_folders() -> Vec<(String, OsString)> {
    watched_folders()
}

pub fn env_resolve(key: &str) -> OsString {
    std::env::var_os(key).unwrap()
}
