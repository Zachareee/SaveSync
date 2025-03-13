use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    sync::{LazyLock, Mutex},
    u8,
};

use tauri_plugin_opener::OpenerExt;

use crate::app_handle;

use super::{
    config_paths::temp,
    fs_utils::resolve_path,
    watch::{upload_file, watch_folder},
    zip_utils::extract,
};

const BUFFERS: LazyLock<Mutex<HashMap<(String, OsString), Vec<u8>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn store_buffer(tag: &str, foldername: &OsStr, buffer: Vec<u8>) {
    BUFFERS
        .lock()
        .unwrap()
        .insert((tag.into(), foldername.into()), buffer);
}

pub fn resolve_conflict((tag, foldername, resolution): (String, OsString, String)) {
    if resolution == "local" {
        upload_file(&tag, &foldername);
        watch_folder(&tag, &foldername);
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

fn retrieve_buffer(tag: &str, foldername: &OsStr) -> Vec<u8> {
    BUFFERS
        .lock()
        .unwrap()
        .remove(&(tag.into(), foldername.into()))
        .unwrap()
}
