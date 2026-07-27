use std::{
    ffi::OsStr,
    time::SystemTime,
};

use serde::Serialize;
use tauri::Emitter;

use crate::app_handle;

pub fn plugin_error(title: &str, description: &str) {
    app_emit("plugin_error", (title, description))
}

pub fn init_result() {
    app_emit("init_result", ());
}

/*
pub fn abort_result(err: &str) {
    app_emit("abort_result", &err);
}
*/

pub fn sync_result(tag: &str, foldername: &OsStr, synced: bool) {
    app_emit("sync_result", (tag, foldername, synced));
}

pub fn conflicting_files(tag: &str, foldername: &OsStr, diff: (SystemTime, SystemTime)) {
    app_emit("conflicting_files", (tag, foldername, diff));
}

fn app_emit<S>(event: &str, payload: S)
where
    S: Serialize + Clone,
{
    app_handle()
        .emit_to("main", event, payload)
        .expect("Unable to emit event")
}
