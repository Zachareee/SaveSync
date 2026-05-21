use crate::{
    app_store, read_app_state,
    savesync::{
        config_paths,
        conflict_files::{resolve_conflict, store_buffer},
        emitter,
        fs_utils::recurse_directories,
        plugin::{FileDetails, Plugin},
        watch::{
            dump_watchers, handle_buffer, strip_zip_extension, toggle_watch, upload_file,
            watch_folder,
        },
    },
    write_app_state,
};
use serde::Deserialize;
use serde_json::from_str;
use std::{
    cmp::{
        max,
        Ordering::{Equal, Greater, Less},
    },
    ffi::{OsStr, OsString},
    thread,
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
        ("conflict_resolve", conflict_resolve_listener),
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
        Ok(mut plugin) => {
            app_store().set_plugin(path);

            if plugin.authenticate() {
                write_app_state(move |s| s.plugin = Some(plugin));
                init_download_folders();
            } else {
                start_server();
                let _ = open_url(
                    plugin.auth_url(&format!("http://localhost:{PORT}")),
                    None::<&str>,
                );
                write_app_state(|s| s.plugin = Some(plugin));
                // emitter::plugin_error(&pathstr, &err);
            };
        }
    }
}

pub fn start_server() {
    tauri_plugin_oauth::start_with_config(
        OauthConfig {
            ports: Some(vec![PORT]),
            response: None,
        },
        move |url| {
            write_app_state(|s| {
                let mut plugin = s.plugin.take().unwrap();
                if let Err(s) = plugin.process_save_credentials(&url) {
                    emitter::plugin_error(plugin.filename().to_str().unwrap(), &s);
                }
                s.plugin = Some(plugin);
            });
            stop_server(PORT);
            init_download_folders();
        },
    )
    .unwrap();
}

fn stop_server(port: u16) {
    tauri_plugin_oauth::cancel(port).unwrap();
}

pub fn init_download_folders() {
    let last_sync = app_store().last_sync();

    thread::spawn(move || {
        if let Some(details) = write_app_state(|s| {
            let plugin = s.plugin_ref();
            match plugin.read_cloud() {
                Ok(details) => {
                    s.tags = details.iter().map(|f| f.tag.clone()).collect();
                    Some(details)
                }
                Err(e) => {
                    emitter::plugin_error("read_cloud", &e);
                    None
                }
            }
        }) {
            details
                .into_iter()
                .for_each(|f| process_cloud_details(f, last_sync));
        }
        app_store().set_last_sync(SystemTime::now())
    });

    emitter::init_result();
}

fn process_cloud_details(
    FileDetails {
        tag,
        folder_name,
        last_modified: cloud_date,
        data,
    }: FileDetails,
    last_sync: SystemTime,
) {
    if let Some(path) = app_store().get_mapping(&tag) {
        let local_date = recurse_directories(
            &path,
            SystemTime::UNIX_EPOCH,
            &mut |_, _, e| e.metadata()?.modified(),
            &max,
        )
        .unwrap_or(SystemTime::UNIX_EPOCH);

        // 6 permutations
        // local < syncd < cloud (Download)
        // cloud < syncd < local (Upload)
        // syncd < local < cloud (Conflict)
        // syncd < cloud < local (Conflict)
        //
        // cloud < local < syncd (Shouldn't be possible)
        // local < cloud < syncd (Shouldn't be possible)

        match (last_sync.cmp(&local_date), last_sync.cmp(&cloud_date)) {
            (k, Less) => {
                println!("Less branch");
                match data.ok_or(|| ()).or_else(|_| {
                    read_app_state(|s| {
                        s.plugin_ref()
                            .download(tag.as_bytes(), folder_name.as_encoded_bytes())
                    })
                }) {
                    Ok(buf) => match k {
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
                        _ => {
                            println!("Extracting");
                            handle_buffer(&path, &folder_name, buf);
                        }
                    },
                    Err(e) => {
                        println!("{e}");
                        emitter::plugin_error("Download", &e);
                        return;
                    }
                }
            }
            (Less, Equal | Greater) => {
                println!("Less with equal or greater");
                upload_file(&tag, path)
            }
            (i, j) => println!("{i:?}, {j:?}"),
        }
        let folder_name = strip_zip_extension(&folder_name);

        watch_folder(&tag, &folder_name);
        emitter::sync_result(&tag, &folder_name.as_os_str(), true);
    }
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

    thread::spawn(move || {
        let bool = toggle_watch(&tag, &foldername);
        if bool {
            upload_file(&tag, &foldername)
        };
        emitter::sync_result(&tag, &foldername, bool);
    });
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

fn conflict_resolve_listener(e: Event) {
    resolve_conflict(from_str(e.payload()).unwrap());
}
