use std::{
    ffi::{OsStr, OsString},
    sync::Mutex,
};

use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

use crate::{app_handle, AppState};

use super::{
    config_paths::temp,
    fs_utils::resolve_path,
    watch::{upload_file, watch_folder},
    zip_utils::extract,
};

pub fn store_buffer(foldername: &OsStr, buffer: Vec<u8>) {
    app_handle()
        .state::<Mutex<AppState>>()
        .lock()
        .unwrap()
        .buffers
        .insert(foldername.into(), buffer);
}

fn retrieve_buffer(foldername: &OsStr) -> Vec<u8> {
    app_handle()
        .state::<Mutex<AppState>>()
        .lock()
        .unwrap()
        .buffers
        .remove(foldername.into())
        .inspect(|x| println!("Got {x:?}"))
        .unwrap()
}

pub fn resolve_conflict(foldername: OsString, resolution: String) {
    if resolution == "local" {
        upload_file(&foldername);
        watch_folder(&foldername);
        return;
    }
    let buf = retrieve_buffer(&foldername);

    if resolution == "cloud" {
        extract(resolve_path(&foldername), buf);
    } else if resolution == "none" {
        let path = temp();
        extract(&path, buf);
        app_handle()
            .opener()
            .open_path(path.to_str().unwrap(), None::<String>)
            .unwrap()
    }
}
