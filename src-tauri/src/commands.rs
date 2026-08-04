use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::command;

use crate::listeners::collect_filter_from_cloud;
use crate::savesync::watch::{drop_watchers, watched_folders};
use crate::savesync::{
    config_paths,
    plugin::{Plugin, PluginInfo},
    store::PathMapping,
};
use crate::{app_store, read_app_state};

#[command]
pub fn get_plugins() -> Vec<PluginInfo> {
    config_paths::get_pluginfiles()
        .into_iter()
        .filter_map(|path| unsafe { Plugin::new(&path) }.and_then(|x| x.info()))
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

#[command]
pub fn add_plugin(filepath: OsString) {
    let path = PathBuf::from(filepath);
    fs::copy(
        &path,
        config_paths::plugin().join(path.file_name().unwrap()),
    )
    .unwrap();
}

#[command]
pub fn logged_in(mut filepath: OsString) -> bool {
    filepath.push(".auth");
    fs::exists(config_paths::creds().join(&filepath)).unwrap_or_default()
}

#[command]
pub fn logout(mut filepath: OsString) {
    filepath.push(".auth");
    let _ = fs::remove_file(config_paths::creds().join(&filepath));
}
