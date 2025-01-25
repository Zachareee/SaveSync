use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub fn get_pluginfiles() -> Vec<PathBuf> {
    fs::read_dir(plugin())
        .unwrap()
        .take_while(Result::is_ok)
        .map(|e| e.unwrap().path())
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
    if !fs::exists(&path).unwrap() {
        fs::create_dir(&path).unwrap()
    }
    path
}
