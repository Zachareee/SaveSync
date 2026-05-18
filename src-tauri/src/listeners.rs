use crate::{
    app_store,
    commands::env_resolve,
    mutate_app_state,
    savesync::{
        config_paths,
        conflict_files::{resolve_conflict, store_buffer},
        emitter,
        fs_utils::FolderItems,
        plugin::{FileDetails, Plugin},
        watch::{dump_watchers, upload_file, watch_folder},
        zip_utils,
    },
};
use serde::Deserialize;
use serde_json::from_str;
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    ffi::{OsStr, OsString},
    fs::read_dir,
    path::Path,
    time::SystemTime,
};
use tauri::{Event, Listener};
use tauri_plugin_oauth::OauthConfig;
use tauri_plugin_opener::open_url;

const PORT: u16 = 3333;

pub fn emit_listeners(app: &tauri::App) {
    let arr: Vec<(&str, fn(Event))> = vec![
        ("init", init_listener),
        // ("abort", abort_listener),
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
    init_func(&from_str::<OsString>(event.payload()).unwrap());
}

// async to prevent UI thread from freezing
pub fn init_func(path: &OsStr) {
    let pathstr = path.to_string_lossy();

    match unsafe { Plugin::new(path) } {
        Err(e) => {
            emitter::plugin_error(&pathstr, &e);
        }
        Ok(plugin) => {
            app_store().set_plugin(path);

            match plugin.authenticate(&format!("http://localhost:{PORT}")) {
                (None, None) => {
                    mutate_app_state(move |s| s.plugin = Some(plugin));
                    let _ = init_download_folders();
                }
                (Some(url), _) => {
                    start_server().unwrap();
                    let _ = open_url(url, None::<&str>);
                    mutate_app_state(|s| s.plugin = Some(plugin));
                    // emitter::plugin_error(&pathstr, &err);
                }
                // this shouldn't be possible
                (_, _) => todo!(),
            };
        }
    }
}

pub fn start_server() -> Result<u16, String> {
    tauri_plugin_oauth::start_with_config(
        OauthConfig {
            ports: Some(vec![PORT]),
            response: None,
        },
        move |url| {
            mutate_app_state(|s| {
                let mut plugin = s.plugin.take().unwrap();
                if let Err(s) = plugin.process_save_credentials(&url) {
                    emitter::plugin_error(plugin.filename().to_str().unwrap(), &s);
                }
                s.plugin = Some(plugin);
                stop_server(PORT);
                let _ = init_download_folders();
            })
        },
    )
    .map_err(|err| err.to_string())
}

fn stop_server(port: u16) {
    let _ = tauri_plugin_oauth::cancel(port);
}

pub fn init_download_folders() -> Result<(), ()> {
    let last_sync = app_store().last_sync();

    emitter::init_result(true);

    mutate_app_state(|s| {
        let plugin = s.plugin.as_ref().unwrap();
        plugin
            .read_cloud()
            .map(|details| {
                s.tags = details.iter().map(|f| f.tag.clone()).collect();
                details
                    .into_iter()
                    .for_each(|f| process_cloud_details(f, last_sync, plugin));
            })
            .map_err(|e| emitter::plugin_error("read_cloud", &e))
    })
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
                    .or_else(|_| plugin.download(tag.as_bytes(), folder_name.as_encoded_bytes()))
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
/*
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
*/

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
        .map(|p| init_func(&p));
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
    env_resolve(env)
        .expect("Environment variable not found")
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

fn oauth_listener(e: Event) {
    let result: Option<(String, String)> =
        mutate_app_state(
            |s| match s.plugin.as_mut()?.process_save_credentials(e.payload()) {
                Ok(_) => None,
                Err(err) => Some((s.plugin.as_ref()?.filename().into_string().unwrap(), err)),
            },
        );
    match result {
        Some((filename, msg)) => {
            emitter::plugin_error(&filename, &msg);
        }
        None => {
            let _ = init_download_folders();
        }
    }
}
