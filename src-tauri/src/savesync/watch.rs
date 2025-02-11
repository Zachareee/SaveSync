use notify_debouncer_full::{new_debouncer, notify::*};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{app_handle, app_state};

use super::{plugin::load_plugin, zip_utils::zip_dir};

fn file_update_callback<P>(tag: &str, path: P)
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    let (zipbuffer, date) = zip_dir(path);
    load_plugin(&app_state(&app_handle()).try_lock().unwrap().plugin.clone())
        .unwrap()
        .upload(tag, path.as_os_str().to_owned(), date, zipbuffer.into());
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

    app_state(&app_handle())
        .lock()
        .unwrap()
        .watchers
        .insert(path, debouncer);
}
