use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::commands::emit_error;

pub fn get_pluginfiles() -> Vec<PathBuf> {
    let path = plugin();
    fs::read_dir(&path)
        .expect(&format!("Unable to read {}", path.to_string_lossy()))
        .filter_map(|dir| dir.ok().map(|result| result.path()))
        .collect()
}

fn appdata() -> PathBuf {
    Path::new(&env::var("APPDATA").expect("Unable to find APPDATA environment variable")).into()
}

fn config() -> PathBuf {
    create_dir_if_not_exist(appdata().join("SaveSync"))
}

fn plugin() -> PathBuf {
    create_dir_if_not_exist(config().join("plugins"))
}

fn create_dir_if_not_exist(path: PathBuf) -> PathBuf {
    if fs::exists(&path).is_ok_and(|x| !x) {
        fs::create_dir(&path).unwrap_or_else(|e| {
            emit_error::<()>(e);
        })
    }
    path
}
