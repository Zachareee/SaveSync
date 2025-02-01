use serde_json::from_str;
use std::ffi::OsString;
use std::rc::Rc;
use std::{collections::HashMap, fmt::Display};

use tauri::{Event, Listener};

use crate::{
    app_handle,
    savesync::{
        config_paths::get_pluginfiles,
        plugin::{load_plugin, Plugin, PluginInfo},
    },
};

pub fn emit_listeners(app: &tauri::App) {
    [("init", init_listener)]
        .into_iter()
        .for_each(|(event, handler)| {
            app.listen(event, handler);
        });
}

fn init_listener(event: Event) {
    let path: OsString = from_str(event.payload()).unwrap();
    let plugins = load_plugins();
    if let Some(x) = plugins.get(&path) {
        println!("Chosen provider: {:?}", x.info().unwrap())
    } else {
        println!("Couldn't find {path:?} in {plugins:?}")
    }
}

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    load_plugins()
        .into_iter()
        .filter_map(|(path, plugin)| {
            plugin.info().map_or_else(
                |e| {
                    emit_error(format!("Failed to run Info() in {:?}: {e}", path));
                    None
                },
                Some,
            )
        })
        .collect()
}

#[tauri::command]
pub fn emit_error<T>(e: T)
where
    T: Display + Sync + Send + 'static,
{
    std::thread::spawn(move || {
        let _ = tauri::WebviewWindowBuilder::new(
            &app_handle(),
            "error",
            tauri::WebviewUrl::App(format!("error.html?msg={e}").into()),
        )
        .title("Error")
        .build();
    });
}

fn load_plugins() -> HashMap<Rc<OsString>, Plugin> {
    get_pluginfiles()
        .into_iter()
        .filter_map(|path| {
            load_plugin(&path).map_or_else(
                |e| {
                    emit_error(e.to_string());
                    None
                },
                |x| Some((x.filename(), x)),
            )
        })
        .collect()
}
