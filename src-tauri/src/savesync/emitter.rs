use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
};

use serde::Serialize;
use tauri::Emitter;

use crate::app_handle;

pub fn plugin_error(title: &str, description: &str) {
    app_emit("plugin_error", (title, description))
}

pub fn init_result(bool: bool) {
    app_emit("init_result", bool);
}

pub fn abort_result(err: &str) {
    app_emit("abort_result", &err);
}

pub fn saved_result() {
    app_emit("saved_result", ())
}

pub fn sync_result(tag: &str, foldername: &OsStr, synced: bool) {
    app_emit("sync_result", (tag, foldername, synced));
}

pub fn filetree_result(map: HashMap<String, Vec<OsString>>) {
    app_emit("filetree_result", map);
}

pub fn conflicting_files(tag: &str, foldername: &OsStr) {
    app_emit("conflicting_files", (tag, foldername));
}

fn app_emit<S>(event: &str, payload: S)
where
    S: Serialize + Clone,
{
    app_handle()
        .emit(event, payload)
        .expect("Unable to emit event")
}
