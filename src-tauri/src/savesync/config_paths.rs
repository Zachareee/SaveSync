use std::{
    ffi::OsString,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use super::fs_utils::FolderItems;

pub fn get_pluginfiles() -> Vec<OsString> {
    let path = plugin();
    path.get_folders()
        .expect(&format!("Unable to read {}", path.to_string_lossy()))
        .iter()
        .map(DirEntry::file_name)
        .collect()
}

#[cfg(not(debug_assertions))]
pub fn appdata() -> PathBuf {
    use std::env;
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

pub fn temp(tag: &str) -> PathBuf {
    create_dir_if_not_exist(config().join("temp").join(tag))
}

fn create_dir_if_not_exist(path: PathBuf) -> PathBuf {
    if fs::exists(&path).is_ok_and(|x| !x) {
        fs::create_dir_all(&path).unwrap()
    }
    path
}
