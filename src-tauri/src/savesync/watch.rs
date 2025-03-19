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
    Mutex<HashMap<(String, OsString), Debouncer<RecommendedWatcher, RecommendedCache>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn upload_file<P>(tag: &str, path: P)
where
    P: AsRef<Path>,
{
    let (zipbuffer, date) = zip_dir(&resolve_path(tag, &path));
    current_plugin()
        .upload(tag, path.as_ref().as_os_str(), date, zipbuffer.into())
        .unwrap();
}

pub fn watch_folder(tag: &str, path: &OsString) -> bool {
    let mut map = WATCHERS.lock().unwrap();
    let key = (tag.to_owned(), path.to_owned());

    // !exist, !initial => add
    // !exist, initial => add
    // exist, !initial => remove
    // exist, initial => nothing

    match map.contains_key(&key) {
        true => {
            map.remove(&key);
            current_plugin().remove(tag, path).unwrap();
            false
        }
        false => {
            let (tag, path) = key.clone();

            let mut debouncer =
                new_debouncer(Duration::from_secs(1), None, move |result| match result {
                    Ok(_) => upload_file(&tag, &path),
                    Err(err) => println!("{err:?}"),
                })
                .unwrap();

            let (tag, path) = key.clone();
            debouncer
                .watch(&resolve_path(&tag, path), RecursiveMode::Recursive)
                .unwrap();

            map.insert(key, debouncer);
            true
        }
    }
}

fn current_plugin() -> Plugin {
    Plugin::new(&app_store().plugin().unwrap()).unwrap()
}

pub fn watched_folders() -> Vec<(String, OsString)> {
    WATCHERS
        .lock()
        .unwrap()
        .iter()
        .map(|((tag, path), _)| (tag.into(), path.into()))
        .collect()
}

pub fn dump_watchers() {
    WATCHERS.lock().unwrap().clear();
}

pub fn drop_watchers(watchers: Vec<(String, OsString)>) {
    let mut map = WATCHERS.lock().unwrap();
    watchers.iter().for_each(|k| {
        map.remove(k);
    });
}
