use crate::{
    app_handle, app_store,
    commands::env_resolve,
    savesync::{
        config_paths, emitter,
        plugin::{FileDetails, Plugin},
        watch::watch_folder,
        zip_utils,
    },
};
use serde::Deserialize;
use serde_json::from_str;
use std::{
    ffi::{OsStr, OsString},
    fs::{read_dir, OpenOptions},
    io::Write,
    path::Path,
    sync::Mutex,
    time::SystemTime,
};
use tauri::{Event, Listener, Manager};

pub fn required_tags() -> Vec<String> {
    app_handle()
        .state::<Mutex<Vec<String>>>()
        .lock()
        .unwrap()
        .to_owned()
}

fn set_required_tags(tags: Vec<String>) {
    *app_handle().state::<Mutex<Vec<String>>>().lock().unwrap() = tags;
}

pub fn emit_listeners(app: &tauri::App) {
    let arr: Vec<(&str, fn(Event))> = vec![
        ("init", init_listener),
        ("abort", abort_listener),
        ("sync", sync_listener),
        ("unload", unload_listener),
        ("saved_plugin", saved_plugin_listener),
    ];
    arr.into_iter().for_each(|(event, handler)| {
        app.listen(event, handler);
    });
}

// wrapper function
fn init_listener(event: Event) {
    if init_func(&from_str::<OsString>(event.payload()).unwrap()) {
        emitter::init_result(true);
    }
}

// async to prevent UI thread from freezing
pub fn init_func(path: &OsStr) -> bool {
    let pathstr = path.to_string_lossy();

    Plugin::new(path).map_or_else(
        |e| {
            emitter::plugin_error(&pathstr, &e);
            false
        },
        |plugin| {
            plugin
                .init()
                .map_err(|e| emitter::plugin_error(&pathstr, &String::from(e)))
                .map(|_| {
                    if let Ok(_) = init_download_folders(&plugin) {
                        app_store().set_plugin(path)
                    }
                })
                .is_ok()
        },
    )
}

fn init_download_folders(plugin: &Plugin) -> Result<(), ()> {
    let last_sync = app_store().last_sync();

    plugin
        .read_cloud()
        .map(|details| {
            let tags = details
                .into_iter()
                .map(|f| process_cloud_details(f, last_sync, plugin))
                .collect();
            set_required_tags(tags);
        })
        .map_err(|e| emitter::plugin_error("read_cloud", &e))
}

fn process_cloud_details(
    FileDetails {
        tag,
        folder_name,
        last_modified: cloud_date,
        data,
    }: FileDetails,
    last_sync: SystemTime,
    plugin: &Plugin,
) -> String {
    if let Some(path) = app_store().get_mapping(&tag) {
        let path = path.join(&folder_name);
        let local_date = get_last_modified(&path).unwrap_or(SystemTime::UNIX_EPOCH);

        if last_sync < cloud_date {
            if local_date < cloud_date {
                match data
                    .ok_or(|| ())
                    .or_else(|_| plugin.download(&tag, &folder_name))
                {
                    Ok(buf) => {
                        zip_utils::extract(&path, buf);
                        watch_folder(&tag, path);
                    }
                    Err(e) => emitter::plugin_error("Download", &e),
                }
            } else {
                // TODO: alert the user to the conflicting data
                // https://github.com/Zachareee/SaveSync/issues/9
            }
        }
    }
    tag
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

/// Fails silently, plugin does not need to implement abort()
/// If a message is returned, it is logged to the logs folder
fn abort_listener(event: Event) {
    let mut filename: OsString = from_str(event.payload()).unwrap();

    if let Some(mut err) = Plugin::new(&filename).map_or(None, |plugin| plugin.abort().err()) {
        emitter::abort_result(&err);

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

    let (env, path) = {
        app_store()
            .path_mapping()
            .get(&tag)
            .expect("Tag name not found")
            .to_owned()
    };
    watch_folder(
        &tag,
        Path::new(&env_resolve(&env)).join(path).join(foldername),
    );
}

fn unload_listener(_: Event) {
    app_store().set_plugin(OsStr::new(""));
}

fn saved_plugin_listener(_: Event) {
    app_store()
        .plugin()
        .filter(|p| !p.is_empty() && config_paths::plugin().join(p).exists())
        .map(|p| {
            if init_func(&p) {
                emitter::saved_result();
            }
        });
}
