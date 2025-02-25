use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::sync::Arc;
use std::{env, path};

use serde::{Deserialize, Serialize};

use crate::app_store;
use crate::listeners::required_tags;
use crate::savesync::config_paths;
use crate::savesync::store::PathMapping;
use crate::savesync::{
    emitter::emit_plugin_error,
    fs_utils::FolderItems,
    plugin::{load_plugin, Plugin, PluginInfo},
};

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    load_plugins()
        .iter()
        .filter_map(|(path, plugin)| {
            plugin.info().map_or_else(
                |e| {
                    emit_plugin_error(
                        &path.to_string_lossy(),
                        &format!("Failed to run Info() in {:?}: {e}", path),
                    );
                    None
                },
                Some,
            )
        })
        .collect()
}

#[tauri::command]
pub fn get_filetree() -> HashMap<String, Vec<OsString>> {
    app_store()
        .path_mapping()
        .iter()
        .map(|(tag, (env, path))| (tag.to_owned(), find_folders_in_path(env, path)))
        .collect()
}

#[tauri::command]
pub fn saved_plugin() -> bool {
    app_store().plugin().is_some_and(|p| p.exists())
}

#[derive(Deserialize, Serialize)]
pub struct Mappings {
    mapping: PathMapping,
    ignored: Vec<String>,
}

#[tauri::command]
pub fn get_mapping() -> Mappings {
    let mapping = app_store().path_mapping();
    let ignored = required_tags()
        .into_iter()
        .filter(|t| !mapping.contains_key(t))
        .collect();
    println!("{:?}", required_tags());
    println!("{ignored:?}");
    Mappings {
        mapping: mapping.clone(),
        ignored,
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

pub fn load_plugins() -> HashMap<Arc<OsString>, Plugin> {
    config_paths::get_pluginfiles()
        .into_iter()
        .filter_map(|path| {
            load_plugin(&path).map_or_else(
                |e| {
                    emit_plugin_error(&path.to_string_lossy(), &e.to_string());
                    None
                },
                |x| Some((x.filename(), x)),
            )
        })
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

pub fn env_resolve(key: &str) -> OsString {
    std::env::var_os(key).unwrap()
}
