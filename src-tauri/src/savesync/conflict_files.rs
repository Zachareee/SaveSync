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

pub fn store_buffer(tag: &str, foldername: &OsStr, buffer: Vec<u8>) {
    app_handle()
        .state::<Mutex<AppState>>()
        .lock()
        .unwrap()
        .buffers
        .insert((tag.into(), foldername.into()), buffer);
}

fn retrieve_buffer(tag: &str, foldername: &OsStr) -> Vec<u8> {
    app_handle()
        .state::<Mutex<AppState>>()
        .lock()
        .unwrap()
        .buffers
        .remove(&(tag.into(), foldername.into()))
        .inspect(|x| println!("Got {x:?}"))
        .unwrap()
}

pub fn resolve_conflict((tag, foldername, resolution): (String, OsString, String)) {
    if resolution == "local" {
        upload_file(&tag, &foldername);
        watch_folder(&tag, &foldername);
        return;
    }
    let buf = retrieve_buffer(&tag, &foldername);

    if resolution == "cloud" {
        extract(resolve_path(&tag, &foldername), buf);
    } else if resolution == "none" {
        let path = temp(&tag);
        extract(&path, buf);
        app_handle()
            .opener()
            .open_path(path.to_str().unwrap(), None::<String>)
            .unwrap()
    }
}
