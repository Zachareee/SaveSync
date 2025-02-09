use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

#[cfg(not(debug_assertions))]
use std::env;

use crate::commands::emit_error;

pub fn get_pluginfiles() -> Vec<PathBuf> {
    let path = plugin();
    fs::read_dir(&path)
        .expect(&format!("Unable to read {}", path.to_string_lossy()))
        .filter_map(|dir| dir.ok().map(|result| result.path()))
        .collect()
}

#[cfg(not(debug_assertions))]
pub fn appdata() -> PathBuf {
    Path::new(&env::var("APPDATA").expect("Unable to find APPDATA environment variable")).into()
}

/// PathBuf pointing to %APPDATA%
#[cfg(debug_assertions)]
pub fn appdata() -> PathBuf {
    Path::new("..").into()
}

/// PathBuf pointing to SaveSync folder in %APPDATA%
pub fn config() -> PathBuf {
    create_dir_if_not_exist(appdata().join("SaveSync"))
}

/// PathBuf pointing to credentials folder in SaveSync
pub fn creds() -> PathBuf {
    create_dir_if_not_exist(config().join("credentials"))
}

/// PathBuf pointing to plugins folder in SaveSync
pub fn plugin() -> PathBuf {
    create_dir_if_not_exist(config().join("plugins"))
}

/// PathBuf pointing to logs folder in SaveSync
pub fn logs() -> PathBuf {
    create_dir_if_not_exist(config().join("logs"))
}

pub fn get_tag_paths() -> Result<HashMap<String, PathBuf>, Box<dyn Error>> {
    serde_json::from_str(&fs::read_to_string(config().join("tagmap.json"))?).map_err(Into::into)
}

fn create_dir_if_not_exist(path: PathBuf) -> PathBuf {
    if fs::exists(&path).is_ok_and(|x| !x) {
        fs::create_dir(&path).unwrap_or_else(|e| {
            emit_error(e.to_string());
        })
    }
    path
}
