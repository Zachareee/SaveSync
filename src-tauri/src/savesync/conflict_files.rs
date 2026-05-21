use std::ffi::{OsStr, OsString};

use tauri_plugin_opener::OpenerExt;

use crate::{app_handle, app_store, savesync::watch::handle_buffer, write_app_state};

use super::{
    config_paths::temp,
    watch::{upload_file, watch_folder},
};

pub fn store_buffer(tag: &str, foldername: &OsStr, buffer: Vec<u8>) {
    write_app_state(|s| s.buffers.insert((tag.into(), foldername.into()), buffer));
}

fn retrieve_buffer(tag: &str, foldername: &OsStr) -> Vec<u8> {
    write_app_state(|s| s.buffers.remove(&(tag.into(), foldername.into())).unwrap())
}

pub fn resolve_conflict((tag, foldername, resolution): (String, OsString, String)) {
    if resolution == "local" {
        upload_file(&tag, &foldername);
        watch_folder(&tag, &foldername);
        return;
    }
    let buf = retrieve_buffer(&tag, &foldername);

    if resolution == "cloud" {
        handle_buffer(app_store().get_mapping(&tag).unwrap().join(&foldername), buf);
    } else if resolution == "none" {
        let path = temp(&tag);
        handle_buffer(&path, buf);
        app_handle()
            .opener()
            .open_path(path.to_str().unwrap(), None::<String>)
            .unwrap()
    }
}
