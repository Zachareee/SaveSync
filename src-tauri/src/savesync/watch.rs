use notify_debouncer_full::{new_debouncer, notify::*, Debouncer, RecommendedCache};
use std::{
    collections::HashMap,
    ffi::OsString,
    path::Path,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use crate::app_store;

use super::{fs_utils::resolve_path, plugin::Plugin, zip_utils::zip_dir};

static WATCHERS: LazyLock<
    Mutex<HashMap<OsString, Debouncer<RecommendedWatcher, RecommendedCache>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn upload_file(path: impl AsRef<Path>) {
    let (zipbuffer, date) = zip_dir(&resolve_path(path.as_ref()));
    current_plugin()
        .upload(
            path.as_ref().as_os_str().as_encoded_bytes(),
            date,
            &zipbuffer,
        )
        .unwrap();
}

pub fn watch_folder(path: &OsString) -> bool {
    let mut map = WATCHERS.lock().unwrap();

    // !exist, !initial => add
    // !exist, initial => add
    // exist, !initial => remove
    // exist, initial => nothing

    match map.contains_key(path) {
        true => {
            map.remove(path);
            current_plugin().remove(path.as_encoded_bytes()).unwrap();
            false
        }
        false => {
            let mut debouncer = {
                let path = path.to_owned();
                new_debouncer(Duration::from_secs(1), None, move |result| match result {
                    Ok(_) => upload_file(&path),
                    Err(err) => println!("{err:?}"),
                })
                .unwrap()
            };

            debouncer
                .watch(&resolve_path(path), RecursiveMode::Recursive)
                .unwrap();

            map.insert(path.to_os_string(), debouncer);
            true
        }
    }
}

fn current_plugin() -> Plugin {
    Plugin::new(&app_store().plugin().unwrap()).unwrap()
}

pub fn watched_folders() -> Vec<OsString> {
    WATCHERS.lock().unwrap().keys().cloned().collect()
}

pub fn dump_watchers() {
    WATCHERS.lock().unwrap().clear();
}

pub fn drop_watchers(watchers: Vec<OsString>) {
    let mut map = WATCHERS.lock().unwrap();
    watchers.iter().for_each(|k| {
        map.remove(k);
    });
}
