use tauri::Emitter;

use crate::app_handle;

pub fn emit_plugin_error(title: &str, description: &str) {
    app_handle()
        .emit("plugin_error", (title, description))
        .unwrap()
}
