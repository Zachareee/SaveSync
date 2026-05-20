use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::command;

use crate::listeners::init_download_folders;
use crate::savesync::fs_utils::FolderItems;
use crate::savesync::watch::{drop_watchers, watched_folders};
use crate::savesync::{
    config_paths, emitter,
    plugin::{Plugin, PluginInfo},
    store::PathMapping,
};
use crate::{app_store, read_app_state};

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
        required: read_app_state(|s| s.tags.clone()),
    }
}

#[command]
pub fn filetree() -> HashMap<String, Vec<OsString>> {
    app_store()
        .path_mapping()
        .into_iter()
        .map(|(tag, path)| (tag, find_folders_in_path(path)))
        .collect()
}

fn find_folders_in_path<T>(path: T) -> Vec<OsString>
where
    T: AsRef<Path>,
{
    path.as_ref()
        .get_folders()
        .unwrap()
        .into_iter()
        .map(|e| e.file_name())
        .collect()
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
    init_download_folders();
}

#[command]
pub fn get_watched_folders() -> Vec<(String, OsString)> {
    watched_folders()
}
