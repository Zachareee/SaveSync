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

pub fn ignored_tags() {
    app_emit("ignored_tags", ());
}

fn app_emit<S>(event: &str, payload: S)
where
    S: Serialize + Clone,
{
    app_handle()
        .emit(event, payload)
        .expect("Unable to emit event")
}
