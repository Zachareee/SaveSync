use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{read_dir, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use std::{env, path};

use serde::Deserialize;
use serde_json::from_str;
use tauri::{Event, Listener};

use crate::savesync::{config_paths, zip_utils};
use crate::savesync::{
    emitter::emit_plugin_error,
    fs_utils::FolderItems,
    plugin::{load_plugin, FileDetails, Plugin, PluginInfo},
    watch::watch_folder,
};
use crate::{app_emit, app_store};

pub fn emit_listeners(app: &tauri::App) {
    let arr: Vec<(&str, fn(Event))> = vec![
        ("init", init_listener),
        ("abort", abort_listener),
        ("sync", sync_listener),
        ("unload", unload_listener),
    ];
    arr.into_iter().for_each(|(event, handler)| {
        app.listen(event, handler);
    });
}

// async to prevent UI thread from freezing
fn init_listener(event: Event) {
    let path: OsString = from_str(event.payload()).unwrap();
    let pathstr = path.to_string_lossy();

    match load_plugins().get(&path) {
        Some(plugin) => {
            let res = plugin
                .init()
                .map_err(|e| emit_plugin_error(&pathstr, &e))
                .is_ok();
            app_emit("init_result", res);
            if !res {
                return;
            }

            init_download_folders(&plugin);
            app_store().set_plugin(config_paths::plugin().join(path));
        }
        None => {
            emit_plugin_error(&pathstr, &format!("{path:?} not found"));
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
    foldername: OsString,
}

fn sync_listener(event: Event) {
    let SyncStruct { tag, foldername } = from_str(event.payload()).unwrap();

    // drop the mutexguard so watch_folder can access mutex later
    let path = {
        app_store()
            .path_mapping()
            .get(&tag)
            .expect("Tag name not found")
            .to_owned()
    };
    watch_folder(&tag, path.join(foldername));
}

fn unload_listener(_: Event) {
    app_store().set_plugin(PathBuf::new());
}

#[tauri::command]
pub fn get_plugins() -> Vec<PluginInfo> {
    load_plugins()
        .iter()
        .filter_map(|(path, plugin)| {
            plugin.info().map_or_else(
                |e| {
                    emit_plugin_error(
                        &path.to_string_lossy(),
                        &format!("Failed to run Info() in {:?}: {e}", path),
                    );
                    None
                },
                Some,
            )
        })
        .collect()
}

#[tauri::command]
pub fn get_filetree() -> HashMap<String, Vec<OsString>> {
    app_store()
        .path_mapping()
        .iter()
        .map(|(tag, path)| (tag.to_owned(), find_folders_in_path(path)))
        .collect()
}

#[tauri::command]
pub fn saved_plugin() -> bool {
    app_store().plugin().is_some_and(|p| p.exists())
}

#[tauri::command]
pub fn get_mapping() -> HashMap<String, OsString> {
    app_store()
        .path_mapping()
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect()
}

#[tauri::command]
pub fn get_envpaths() -> HashMap<String, OsString> {
    env::vars()
        .filter_map(|(k, v)| {
            path::absolute(&v)
                .ok()
                .filter(|p| p.exists())
                .map(|p| (k, p.into_os_string()))
        })
        .collect()
}

pub fn load_plugins() -> HashMap<Arc<OsString>, Plugin> {
    config_paths::get_pluginfiles()
        .into_iter()
        .filter_map(|path| {
            load_plugin(&path).map_or_else(
                |e| {
                    emit_plugin_error(&path.to_string_lossy(), &e.to_string());
                    None
                },
                |x| Some((x.filename(), x)),
            )
        })
        .collect()
}

fn init_download_folders(plugin: &Plugin) {
    let last_sync = app_store().last_sync();

    // TODO: change unwrap to handle error
    plugin.read_cloud().unwrap().into_iter().for_each(
        |FileDetails {
             tag,
             folder_name,
             last_modified: cloud_date,
             data,
         }| {
            // TODO: change unwrap to a file selection prompt
            // https://github.com/Zachareee/SaveSync/issues/3
            let path = app_store().get_mapping(&tag).unwrap().join(&folder_name);
            let local_date = get_last_modified(&path).unwrap_or(SystemTime::UNIX_EPOCH);

            if last_sync < cloud_date {
                if local_date < cloud_date {
                    zip_utils::extract(
                        &path,
                        // TODO: change unwrap to handle error
                        data.unwrap_or_else(|| plugin.download(&tag, &folder_name).unwrap()),
                    )
                } else {
                    // TODO: alert the user to the conflicting data
                    // https://github.com/Zachareee/SaveSync/issues/9
                }
            }
            watch_folder(&tag, path);
        },
    );
}

fn get_last_modified<T>(path: T) -> std::io::Result<SystemTime>
where
    T: AsRef<Path>,
{
    read_dir(&path)?.try_fold(SystemTime::UNIX_EPOCH, |accum, entry| {
        let entry = entry.unwrap();
        let timestamp = if entry.file_type().unwrap().is_dir() {
            get_last_modified(&path.as_ref().join(entry.file_name()))?
        } else {
            entry.metadata()?.modified()?
        };

        Ok(if accum < timestamp { timestamp } else { accum })
    })
}

fn find_folders_in_path<T>(path: T) -> Vec<OsString>
where
    T: AsRef<Path>,
{
    path.as_ref()
        .get_folders()
        .unwrap()
        .into_iter()
        .map(|e| e.file_name())
        .collect()
}
