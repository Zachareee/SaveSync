use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::{env, path};

use serde::{Deserialize, Serialize};
use tauri::command;

use crate::listeners::init_download_folders;
use crate::savesync::watch::{drop_watchers, watched_folders};
use crate::savesync::{
    config_paths, emitter,
    plugin::{Plugin, PluginInfo},
    store::PathMapping,
};
use crate::{app_store, mutate_app_state};

#[command]
pub fn get_plugins() -> Vec<PluginInfo> {
    config_paths::get_pluginfiles()
        .into_iter()
        .filter_map(|path| {
            unsafe { Plugin::new(&path) }.map_or_else(
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Mappings {
    mapping: PathMapping,
    required: Vec<String>,
}

#[command]
pub fn get_mapping() -> Mappings {
    Mappings {
        mapping: app_store().path_mapping(),
        required: mutate_app_state(|s| s.tags.clone()),
    }
}

#[command]
pub fn set_mapping(map: PathMapping) {
    drop_watchers(
        watched_folders()
            .into_iter()
            .filter(|(k, _)| !map.contains_key(k))
            .collect(),
    );
    app_store().set_mapping(map);
    init_download_folders().unwrap()
}

#[command]
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

#[command]
pub fn get_watched_folders() -> Vec<(String, OsString)> {
    watched_folders()
}

pub fn env_resolve(key: &str) -> Result<PathBuf, String> {
    match std::env::var_os(key) {
        Some(osstr) => Path::new(&osstr).canonicalize().map_err(|e| e.to_string()),
        None => Err("Env key not found".to_string()),
    }
}
