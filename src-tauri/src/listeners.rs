use crate::{
    app_handle, app_store,
    commands::env_resolve,
    savesync::{
        config_paths,
        conflict_files::{resolve_conflict, store_buffer},
        emitter,
        fs_utils::FolderItems,
        plugin::{FileDetails, Plugin},
        watch::{dump_watchers, upload_file, watch_folder},
        zip_utils,
    },
    AppState,
};
use serde::Deserialize;
use serde_json::from_str;
use std::{
    cmp::Ordering::{Equal, Greater, Less},
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
        .state::<Mutex<AppState>>()
        .lock()
        .unwrap()
        .tags
        .clone()
}

fn set_required_tags(tags: Vec<String>) {
    app_handle().state::<Mutex<AppState>>().lock().unwrap().tags = tags;
}

pub fn emit_listeners(app: &tauri::App) {
    let arr: Vec<(&str, fn(Event))> = vec![
        ("init", init_listener),
        ("abort", abort_listener),
        ("sync", sync_listener),
        ("unload", unload_listener),
        ("saved_plugin", saved_plugin_listener),
        ("filetree", filetree_listener),
        ("conflict_resolve", conflict_resolve_listener),
    ];
    arr.into_iter().for_each(|(event, handler)| {
        app.listen(event, handler);
    });
}

// wrapper function
fn init_listener(event: Event) {
    emitter::init_result(init_func(&from_str::<OsString>(event.payload()).unwrap()));
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

pub fn init_download_folders(plugin: &Plugin) -> Result<(), ()> {
    let last_sync = app_store().last_sync();

    plugin
        .read_cloud()
        .map(|details| {
            set_required_tags(details.iter().map(|f| f.tag.clone()).collect());
            details
                .into_iter()
                .for_each(|f| process_cloud_details(f, last_sync, plugin));
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
) {
    if let Some(path) = app_store().get_mapping(&tag) {
        let path = path.join(&folder_name);

        let local_date = get_last_modified(&path).unwrap_or(SystemTime::UNIX_EPOCH);

        // 6 permutations
        // local < syncd < cloud (Download)
        // cloud < syncd < local (Upload)
        // syncd < local < cloud (Conflict)
        // syncd < cloud < local (Conflict)
        //
        // cloud < local < syncd (Shouldn't be possible)
        // local < cloud < syncd (Shouldn't be possible)

        match (last_sync.cmp(&local_date), last_sync.cmp(&cloud_date)) {
            (Equal, _) | (_, Equal) | (Greater, Greater) => (),
            (k, Less) => {
                println!("Less branch");
                match data
                    .ok_or(|| ())
                    .or_else(|_| plugin.download(&tag, &folder_name))
                {
                    Ok(buf) => match k {
                        Greater => {
                            println!("Extracting");
                            zip_utils::extract(&path, buf)
                        }
                        Less => {
                            println!("Both less");
                            store_buffer(&tag, &folder_name, buf);
                            emitter::conflicting_files(
                                &tag,
                                &folder_name,
                                (local_date, cloud_date),
                            );
                            return;
                        }
                        _ => (),
                    },
                    Err(e) => {
                        println!("{e}");
                        emitter::plugin_error("Download", &e);
                        return;
                    }
                }
            }
            (Less, Greater) => upload_file(&tag, path),
        }
        watch_folder(&tag, &folder_name);
    }
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

    upload_file(&tag, &foldername);
    emitter::sync_result(&tag, &foldername, watch_folder(&tag, &foldername));
}

fn unload_listener(_: Event) {
    app_store().set_plugin(OsStr::new(""));
    dump_watchers();
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

fn filetree_listener(_: Event) {
    emitter::filetree_result(
        app_store()
            .path_mapping()
            .into_iter()
            .map(|(tag, (env, path))| (tag, find_folders_in_path(&env, path)))
            .collect(),
    )
}

fn find_folders_in_path<T>(env: &str, path: T) -> Vec<OsString>
where
    T: AsRef<Path>,
{
    Path::new(&env_resolve(env))
        .join(path)
        .get_folders()
        .unwrap()
        .into_iter()
        .map(|e| e.file_name())
        .collect()
}

fn conflict_resolve_listener(e: Event) {
    resolve_conflict(from_str(e.payload()).unwrap());
}
