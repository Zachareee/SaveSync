use std::{collections::HashMap, error::Error, fs, path::PathBuf};

use notify_debouncer_full::{notify::RecommendedWatcher, Debouncer, RecommendedCache};

use crate::{app_handle, app_state};

use super::config_paths;

#[allow(dead_code)]
pub struct AppState {
    pub plugin: PathBuf,
    pub path_mapping: HashMap<String, PathBuf>,
    pub watchers: HashMap<PathBuf, Debouncer<RecommendedWatcher, RecommendedCache>>,
}

pub fn read_state() -> AppState {
    AppState {
        plugin: fs::read_to_string(config_paths::config().join("last_plugin.txt"))
            .unwrap_or_default()
            .into(),
        path_mapping: get_tag_paths().unwrap_or_default(),
        watchers: Default::default(),
    }
}

pub fn save_state() {
    let handle = &app_handle();
    let binding = app_state(&handle);
    let state = binding.lock().unwrap();
    let plugin = &state.plugin;
    let path_mapping = &state.path_mapping;

    fs::write(tagmap_loc(), serde_json::to_string(&path_mapping).unwrap()).unwrap();
    fs::write(plugin_loc(), plugin.as_os_str().as_encoded_bytes()).unwrap()
}

fn tagmap_loc() -> PathBuf {
    config_paths::config().join("tagmap.json")
}

fn plugin_loc() -> PathBuf {
    config_paths::config().join("last_plugin.txt")
}

fn get_tag_paths() -> Result<HashMap<String, PathBuf>, Box<dyn Error>> {
    serde_json::from_str(&fs::read_to_string(
        config_paths::config().join("tagmap.json"),
    )?)
    .map_err(Into::into)
}
