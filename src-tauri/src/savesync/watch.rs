use notify_debouncer_full::{new_debouncer, notify::*, Debouncer, RecommendedCache};
use std::{
    collections::HashMap,
    ffi::OsString,
    path::Path,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use crate::{app_store, mutate_app_state};

use super::zip_utils::zip_dir;

static WATCHERS: LazyLock<
    Mutex<HashMap<(String, OsString), Debouncer<RecommendedWatcher, RecommendedCache>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn upload_file<P>(tag: &str, path: P)
where
    P: AsRef<Path>,
{
    let (zipbuffer, date) = zip_dir(&app_store().get_mapping(tag).unwrap().join(&path));
    mutate_app_state(|s| {
        s.plugin_ref()
            .upload(
                tag.as_bytes(),
                path.as_ref().file_name().unwrap().as_encoded_bytes(),
                date,
                zipbuffer.as_slice(),
            )
            .unwrap()
    });
}

fn setup_watcher(key: (String, OsString)) -> Debouncer<RecommendedWatcher, RecommendedCache> {
    let (tag, path) = key.clone();
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, move |result| match result {
        Ok(_) => upload_file(&tag, &path),
        Err(err) => println!("{err:?}"),
    })
    .unwrap();

    let (tag, path) = key.clone();

    debouncer
        .watch(&app_store().get_mapping(&tag).unwrap().join(path), RecursiveMode::Recursive)
        .unwrap();

    debouncer
}
pub fn watch_folder(tag: &str, path: &OsString) {
    mutate_watchers(|map| {
        let key = (tag.to_owned(), path.to_owned());
        if !map.contains_key(&key) {
            map.insert(key.clone(), setup_watcher(key));
        }
    })
}

pub fn toggle_watch(tag: &str, path: &OsString) -> bool {
    mutate_watchers(|map| {
        let key = (tag.to_owned(), path.to_owned());

        // !exist, !initial => add
        // !exist, initial => add
        // exist, !initial => remove
        // exist, initial => nothing

        match map.contains_key(&key) {
            true => {
                map.remove(&key);
                mutate_app_state(|s| {
                    s.plugin_ref()
                        .remove(tag.as_bytes(), path.as_encoded_bytes())
                        .unwrap()
                });
                false
            }
            false => {
                map.insert(key.clone(), setup_watcher(key));
                true
            }
        }
    })
}

pub fn watched_folders() -> Vec<(String, OsString)> {
    mutate_watchers(|map| {
        map.iter()
            .map(|((tag, path), _)| (tag.into(), path.into()))
            .collect()
    })
}

pub fn dump_watchers() {
    mutate_watchers(|map| map.clear());
}

pub fn drop_watchers(watchers: Vec<(String, OsString)>) {
    mutate_watchers(|map| {
        watchers.iter().for_each(|k| {
            map.remove(k);
        })
    });
}

pub fn mutate_watchers<F, T>(func: F) -> T
where
    F: FnOnce(
        &mut HashMap<(String, OsString), Debouncer<RecommendedWatcher, RecommendedCache>>,
    ) -> T,
{
    func(&mut WATCHERS.lock().unwrap())
}
