use std::ffi::{OsStr, OsString};

use tauri_plugin_opener::OpenerExt;

use crate::{app_handle, app_store, mutate_app_state};

use super::{
    config_paths::temp,
    watch::{upload_file, watch_folder},
    zip_utils::extract,
};

pub fn store_buffer(tag: &str, foldername: &OsStr, buffer: Vec<u8>) {
    mutate_app_state(|s| s.buffers.insert((tag.into(), foldername.into()), buffer));
}

fn retrieve_buffer(tag: &str, foldername: &OsStr) -> Vec<u8> {
    mutate_app_state(|s| s.buffers.remove(&(tag.into(), foldername.into())).unwrap())
}

pub fn resolve_conflict((tag, foldername, resolution): (String, OsString, String)) {
    if resolution == "local" {
        upload_file(&tag, &foldername);
        watch_folder(&tag, &foldername);
        return;
    }
    let buf = retrieve_buffer(&tag, &foldername);

    if resolution == "cloud" {
        extract(app_store().get_mapping(&tag).unwrap().join(&foldername), buf).unwrap();
    } else if resolution == "none" {
        let path = temp(&tag);
        extract(&path, buf).unwrap();
        app_handle()
            .opener()
            .open_path(path.to_str().unwrap(), None::<String>)
            .unwrap()
    }
}
