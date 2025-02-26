use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::{env, path};

use serde::{Deserialize, Serialize};

use crate::app_store;
use crate::listeners::required_tags;
use crate::savesync::{
    config_paths,
    emitter::emit_plugin_error,
    fs_utils::FolderItems,
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
                    emit_plugin_error(&path.to_string_lossy(), &e.to_string());
                    None
                },
                |x| {
                    x.info()
                        .map_err(|e| {
                            emit_plugin_error(
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

#[tauri::command]
pub fn get_filetree() -> HashMap<String, Vec<OsString>> {
    app_store()
        .path_mapping()
        .into_iter()
        .map(|(tag, (env, path))| (tag, find_folders_in_path(&env, path)))
        .collect()
}

fn find_folders_in_path<T>(env: &str, path: T) -> Vec<OsString>
where
    T: AsRef<Path>,
{
    Path::new(&env_resolve(env))
        .join(path)
        .get_folders()
        .unwrap()
        .into_iter()
        .map(|e| e.file_name())
        .collect()
}

#[derive(Deserialize, Serialize)]
pub struct Mappings {
    mapping: PathMapping,
    ignored: Vec<String>,
}

#[tauri::command]
pub fn get_mapping() -> Mappings {
    let mapping = app_store().path_mapping();
    Mappings {
        mapping: mapping.clone(),
        ignored: required_tags()
            .into_iter()
            .filter(|t| !mapping.contains_key(t))
            .collect(),
    }
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

pub fn env_resolve(key: &str) -> OsString {
    std::env::var_os(key).unwrap()
}
