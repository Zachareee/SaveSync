use std::collections::HashMap;
use std::path::PathBuf;

use crate::savesync::{
    config_paths::get_pluginfiles,
    plugin::{load_plugin, Plugin, PluginInfo},
};

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    load_plugins()
        .into_iter()
        .map(|(_, plugin)| plugin.info().unwrap())
        .collect()
}

fn load_plugins() -> HashMap<PathBuf, Plugin> {
    get_pluginfiles()
        .into_iter()
        .map(|f| (f.clone(), load_plugin(&f).unwrap()))
        .collect()
}
