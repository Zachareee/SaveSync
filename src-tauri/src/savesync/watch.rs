use notify_debouncer_full::{new_debouncer, notify::*, Debouncer, RecommendedCache};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::Duration,
};

use crate::app_store;

use super::{plugin::load_plugin, zip_utils::zip_dir};

static WATCHERS: LazyLock<
    Mutex<HashMap<PathBuf, Debouncer<RecommendedWatcher, RecommendedCache>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn file_update_callback<P>(tag: &str, path: P)
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    let (zipbuffer, date) = zip_dir(path);
    load_plugin(&app_store().plugin().unwrap()).unwrap().upload(
        tag,
        path.as_os_str().to_owned(),
        date,
        zipbuffer.into(),
    );
}

pub fn watch_folder(tag: &str, path: PathBuf) {
    let cloned = path.clone();
    let tag = tag.to_owned();

    let mut debouncer = new_debouncer(Duration::from_secs(1), None, move |result| match result {
        Ok(_) => file_update_callback(&tag, &cloned),
        Err(err) => println!("{err:?}"),
    })
    .unwrap();

    debouncer.watch(&path, RecursiveMode::Recursive).unwrap();

    WATCHERS.lock().unwrap().insert(path, debouncer);
}
