use std::{env, fs, path::PathBuf};

use crate::service::{plugin_info, Service};

#[tauri::command]
pub fn get_services() -> Vec<Service> {
    get_pluginfiles()
        .into_iter()
        .map(|f| {
            plugin_info(&f.to_str().unwrap()).expect(&format!(
                "Unable to call Info() for selected plugin {}",
                f.file_name().unwrap().to_string_lossy()
            ))
        })
        .collect()
}

fn get_pluginfiles() -> Vec<PathBuf> {
    fs::read_dir(get_pluginpath())
        .unwrap()
        .take_while(Result::is_ok)
        .map(|e| e.unwrap().path())
        .collect()
}

fn get_configpath() -> String {
    let path = format!(
        "{}/com.savesync.app",
        env::var("APPDATA").expect("Unable to find APPDATA environment variable")
    );
    create_dir_if_not_exist(&path)
}

fn get_pluginpath() -> String {
    create_dir_if_not_exist(&format!("{}/plugins", get_configpath())).to_string()
}

fn create_dir_if_not_exist(path: &str) -> String {
    if !fs::exists(path).unwrap() {
        fs::create_dir(path).unwrap()
    }
    path.to_string()
}
