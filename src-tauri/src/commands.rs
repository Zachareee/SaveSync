use std::ffi::OsString;
use std::fs::{read_dir, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};

use serde::Deserialize;
use serde_json::from_str;
use tauri::{Event, Listener};

use crate::savesync::config_paths;
use crate::savesync::{
    plugin::{load_plugin, Plugin, PluginInfo},
    watch::watch_folder,
};
use crate::{app_emit, app_handle, app_state};

pub fn emit_listeners(app: &tauri::App) {
    let arr: Vec<(&str, fn(Event))> = vec![
        ("init", init_listener),
        ("abort", abort_listener),
        ("sync", sync_listener),
    ];
    arr.into_iter().for_each(|(event, handler)| {
        app.listen(event, handler);
    });
}

// async to prevent UI thread from freezing
fn init_listener(event: Event) {
    let path: OsString = from_str::<OsString>(event.payload()).unwrap();

    match load_plugins().get(&path) {
        Some(plugin) => {
            let res = plugin.init().map_err(|e| emit_error(e)).is_ok();
            app_emit("init_result", res);
            if res {
                app_state(&app_handle()).lock().unwrap().plugin = config_paths::plugin().join(path)
            }
        }
        None => {
            emit_error(format!("{path:?} not found"));
        }
    }
}

/// Fails silently, plugin does not need to implement abort()
/// If a message is returned, it is logged to the logs folder
fn abort_listener(event: Event) {
    let mut filename: OsString = from_str::<OsString>(event.payload()).unwrap();

    if let Some(mut err) = load_plugins()
        .get(&filename)
        .map_or(None, |plugin| plugin.abort().err())
    {
        app_emit("abort_result", &err);

        filename.push(".txt");

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(config_paths::logs().join(filename))
        {
            err.push('\n');
            let _ = file.write_all(&err.into_bytes());
        }
    }
}

#[derive(Deserialize)]
struct SyncStruct {
    tag: String,
    foldername: String,
}

fn sync_listener(event: Event) {
    let SyncStruct { tag, foldername } = from_str(event.payload()).unwrap();

    // drop the mutexguard so watch_folder can access mutex later
    let path = {
        app_state(&app_handle())
            .lock()
            .unwrap()
            .path_mapping
            .get(&tag)
            .expect("Tag name not found")
            .to_owned()
    };
    watch_folder(path.join(foldername));
}

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    load_plugins()
        .iter()
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
pub fn get_fmap() -> HashMap<String, Vec<OsString>> {
    app_state(&app_handle())
        .lock()
        .unwrap()
        .path_mapping
        .iter()
        .map(|(tag, path)| (tag.to_owned(), find_folders_in_path(path)))
        .collect()
}

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

pub fn load_plugins() -> HashMap<Arc<OsString>, Plugin> {
    config_paths::get_pluginfiles()
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

fn find_folders_in_path<T>(path: T) -> Vec<OsString>
where
    T: AsRef<Path>,
{
    read_dir(path)
        .unwrap()
        .filter_map(|r| {
            r.ok().and_then(|entry| {
                if entry.file_type().is_ok_and(|filetype| filetype.is_dir()) {
                    Some(entry.file_name())
                } else {
                    None
                }
            })
        })
        .collect()
}
