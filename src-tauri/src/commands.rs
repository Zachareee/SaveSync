use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::DirEntry;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::command;

use crate::listeners::collect_filter_from_cloud;
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
                    emitter::plugin_error(&path, &e.to_string());
                    None
                },
                |x| {
                    x.info()
                        .map_err(|e| {
                            emitter::plugin_error(
                                &path,
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
    required: HashSet<String>,
}

#[command]
pub fn get_mapping() -> Mappings {
    Mappings {
        mapping: app_store().path_mapping(),
        required: read_app_state(|s| s.tags.clone()),
    }
}

#[command]
pub fn filetree() -> HashMap<String, Vec<(OsString, bool)>> {
    app_store()
        .path_mapping()
        .into_iter()
        .map(|(tag, path)| (tag, find_folders_in_path(path)))
        .collect()
}

fn find_folders_in_path<T>(path: T) -> Vec<(OsString, bool)>
where
    T: AsRef<Path>,
{
    path.as_ref()
        .read_dir()
        .unwrap()
        .collect::<io::Result<Vec<DirEntry>>>()
        .unwrap()
        .iter()
        .map(|e| (e.file_name(), e.metadata().unwrap().is_dir()))
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
    let store = app_store();
    let keys: HashSet<String> = map
        .iter()
        .filter(|(k, v)| store.get_mapping(k).is_none_or(|p| p != **v))
        .map(|(k, _)| k.to_owned())
        .collect();
    store.set_mapping(map);
    collect_filter_from_cloud(move |key| keys.contains(key));
}

#[command]
pub fn get_watched_folders() -> Vec<(String, OsString)> {
    watched_folders()
}
