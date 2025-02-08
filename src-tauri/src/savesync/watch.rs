use notify_debouncer_full::{new_debouncer, notify::*, Debouncer, RecommendedCache};
use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read, Seek, Write},
    path::PathBuf,
    sync::{LazyLock, Mutex},
    time::Duration,
};
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{app_handle, app_state};

use super::plugin::load_plugin;

static MUTEX: LazyLock<Mutex<HashMap<PathBuf, Debouncer<RecommendedWatcher, RecommendedCache>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn file_update_callback(path: &PathBuf) {
    println!("{path:?} was updated");
    let buffer = Cursor::new(vec![]);
    let mut zip = ZipWriter::new(buffer);

    println!("Zipping");
    recurse_zip_file(&mut zip, path, &PathBuf::new());
    println!("Zipping done");

    println!("Syncing");
    let buffer = zip
        .finish_into_readable()
        .unwrap()
        .into_inner()
        .bytes()
        .filter_map(|r| r.ok())
        .collect();
    println!("Processing zip buffer");
    let plugin = app_state(&app_handle()).try_lock().unwrap().plugin.clone();
    println!("Plugin var: {plugin:?}");
    load_plugin(&plugin).unwrap().sync(buffer);
    println!("Sync done");
}

fn recurse_zip_file<T>(zip: &mut ZipWriter<T>, path: &PathBuf, relative_path: &PathBuf)
where
    T: Write + Seek,
{
    println!("zipping {path:?}");
    fs::read_dir(path).unwrap().into_iter().for_each(|f| {
        let entry = f.unwrap();
        let filename = entry.file_name();

        if entry.file_type().unwrap().is_dir() {
            recurse_zip_file(
                zip,
                &path.join(entry.file_name()),
                &relative_path.join(filename),
            );
        } else {
            zip.start_file_from_path(relative_path.join(filename), SimpleFileOptions::default())
                .unwrap();
        }
    });
}

pub fn watch_folder(path: PathBuf) {
    println!("Watching {path:?}");
    let cloned = path.clone();

    let mut debouncer = new_debouncer(Duration::from_secs(1), None, move |result| match result {
        Ok(_) => file_update_callback(&cloned),
        Err(err) => println!("{err:?}"),
    })
    .unwrap();

    debouncer.watch(&path, RecursiveMode::Recursive).unwrap();

    MUTEX
        .lock()
        .unwrap()
        //app_state(&app_handle())
        //    .lock()
        //    .unwrap()
        //    .watchers
        .insert(path, debouncer);
}
