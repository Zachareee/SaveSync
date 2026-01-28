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
    AppState, REDIRECT_URL,
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
use tauri_plugin_opener::open_url;

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
        ("oauth_redirect", oauth_listener),
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

    match Plugin::new(path) {
        Err(e) => {
            emitter::plugin_error(&pathstr, &e);
            false
        }
        Ok(plugin) => {
            app_store().set_plugin(path);

            match plugin.validate(REDIRECT_URL) {
                (None, None) => {
                    let _ = init_download_folders(&plugin);
                    true
                }
                (Some(url), Some(err)) => {
                    open_url(url, None::<&str>);
                    emitter::plugin_error(&pathstr, &err);
                    false
                }
                (_, _) => todo!(),
            }
        }
    }
}

pub fn init_download_folders(plugin: &Plugin) -> Result<(), ()> {
    let last_sync = app_store().last_sync();

    plugin
        .read_cloud()
        .map(|details| {
            details
                .into_iter()
                .for_each(|f| process_cloud_details(f, last_sync, plugin));
        })
        .map_err(|e| emitter::plugin_error("read_cloud", &e))
}

fn process_cloud_details(
    FileDetails {
        folder_name,
        last_modified: cloud_date,
    }: FileDetails,
    last_sync: SystemTime,
    plugin: &Plugin,
) {
    match app_store().get_mapping(&folder_name) {
        Some(path) => {
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
                    match plugin.download(folder_name.as_encoded_bytes()) {
                        Ok(buf) => match k {
                            Greater => {
                                println!("Extracting");
                                zip_utils::extract(&path, buf)
                            }
                            Less => {
                                println!("Both less");
                                store_buffer(&folder_name, buf);
                                emitter::conflicting_files(&folder_name, (local_date, cloud_date));
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
                (Less, Greater) => upload_file(path),
            }
            watch_folder(&folder_name);
        }
        None => todo!(),
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

    if let Err(err) = Plugin::new(&filename).map_or(Ok(()), |plugin| plugin.abort()) {
        emitter::abort_result(&err);

        filename.push(".txt");

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(config_paths::logs().join(filename))
        {
            let _ = file.write_all(&err.as_bytes());
        }
    }
}

// #[derive(Deserialize)]
// struct SyncStruct {
//     tag: String,
//     foldername: OsString,
// }

fn sync_listener(event: Event) {
    let foldername: OsString = from_str(event.payload()).unwrap();

    upload_file(&foldername);
    emitter::sync_result(&foldername, watch_folder(&foldername));
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
    emitter::filetree_result(app_store().path_mapping())
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
    let (foldername, resolution): (OsString, String) = from_str(e.payload()).unwrap();
    resolve_conflict(foldername, resolution);
}

fn oauth_listener(e: Event) {
    let mut plugin = Plugin::new(&app_store().plugin().unwrap()).unwrap();
    match plugin.process_save_credentials(e.payload()) {
        Ok(_) => {
            let _ = init_download_folders(&plugin);
        }
        Err(err) => emitter::plugin_error(&plugin.filename().to_string_lossy(), &err),
    }
}
