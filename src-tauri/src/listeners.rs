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
        Ordering::{Equal, Greater, Less},
        max,
    },
    ffi::{OsStr, OsString},
    ops::RangeInclusive,
    thread,
    time::SystemTime,
};
use tauri::{Event, Listener};
use tauri_plugin_oauth::OauthConfig;
use tauri_plugin_opener::open_url;

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
    if let Some(mut plugin) = unsafe { Plugin::new(path) } {
        app_store().set_plugin(path);

        if plugin.authenticate() {
            write_app_state(move |s| s.plugin = Some(plugin));
            init_download_folders();
        } else {
            let port = start_server();
            if let Some(url) = plugin.auth_url(&format!("http://localhost:{port}")) {
                let _ = open_url(url, None::<&str>);
                write_app_state(|s| {
                    s.plugin = Some(plugin);
                    s.server_port = Some(port);
                });
            }
        };
    }
}

const PORTS: RangeInclusive<u16> = 5000..=5009;

pub fn start_server() -> u16 {
    tauri_plugin_oauth::start_with_config(
        OauthConfig {
            redirect_uri: None,
            ports: Some(PORTS.collect()),
            response: None,
        },
        move |url| {
            let result = write_app_state(|s| {
                let mut plugin = s.plugin.take().unwrap();
                let result = plugin.process_save_credentials(&url);
                if result {
                    s.plugin = Some(plugin);
                }
                result
            });
            stop_server();
            if result {
                init_download_folders();
            }
        },
    )
    .unwrap()
}

fn stop_server() {
    write_app_state(|s| tauri_plugin_oauth::cancel(s.server_port.take().unwrap()).unwrap());
}

pub fn init_download_folders() {
    collect_filter_from_cloud(|_| true);
    emitter::init_result();
}

pub fn collect_filter_from_cloud<F>(lambda: F) -> thread::JoinHandle<()>
where
    F: Fn(&String) -> bool + Send + 'static,
{
    let last_sync = app_store().last_sync();

    thread::spawn(move || {
        if let Some(details) = write_app_state(|s| {
            let plugin = s.plugin_mut_ref();
            plugin.read_cloud().inspect(|details| {
                s.tags = details.iter().map(|f| f.tag.clone()).collect();
            })
        }) {
            details
                .into_iter()
                .filter(|FileDetails { tag, .. }| lambda(tag))
                .for_each(|f| process_cloud_details(f, last_sync));
        }
    })
}

fn process_cloud_details(
    FileDetails {
        tag,
        folder_name: item,
        last_modified: cloud_date,
        data,
    }: FileDetails,
    last_sync: SystemTime,
) {
    if let Some(path) = app_store().get_mapping(&tag) {
        let fileinfo = strip_zip_extension(&item);

        let local_date = recurse_directories(
            &path.join(fileinfo.value()),
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
                if let Some(buf) = data.or_else(|| {
                    read_app_state(|s| {
                        s.plugin_ref()
                            .download(tag.as_bytes(), item.as_encoded_bytes())
                    })
                }) {
                    match k {
                        Less => {
                            println!("Both less");
                            store_buffer(&tag, &item, buf);
                            emitter::conflicting_files(&tag, &item, (local_date, cloud_date));
                            return;
                        }
                        _ => {
                            println!("Extracting");
                            handle_buffer(&path, &fileinfo, buf);
                        }
                    }
                }
            }
            (Less, Equal | Greater) => {
                println!("Less with equal or greater");
                upload_file(&tag, &fileinfo)
            }
            (i, j) => println!("{i:?}, {j:?}"),
        }

        let val = fileinfo.value();
        watch_folder(&tag, &val);
        emitter::sync_result(&tag, &val.as_os_str(), true);
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
            upload_file(&tag, &strip_zip_extension(&foldername))
        };
        emitter::sync_result(&tag, &foldername, bool);
    });
}

fn unload_listener(_: Event) {
    app_store().set_plugin(OsStr::new(""));
    write_app_state(|s| s.plugin = None);
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
