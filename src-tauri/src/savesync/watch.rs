use notify_debouncer_full::{new_debouncer, notify::*};
use std::{
    fs,
    io::{Cursor, Read, Seek, Write},
    path::PathBuf,
    time::Duration,
};
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{app_handle, app_state};

use super::plugin::load_plugin;

fn file_update_callback(path: &PathBuf) {
    let buffer = Cursor::new(vec![]);
    let mut zip = ZipWriter::new(buffer);

    recurse_zip_file(&mut zip, path, &PathBuf::new());

    load_plugin(&app_state(&app_handle()).try_lock().unwrap().plugin.clone())
        .unwrap()
        .sync(
            zip.finish_into_readable()
                .unwrap()
                .into_inner()
                .into_inner(),
        );
}

fn recurse_zip_file<T>(zip: &mut ZipWriter<T>, path: &PathBuf, relative_path: &PathBuf)
where
    T: Write + Seek,
{
    fs::read_dir(path).unwrap().into_iter().for_each(|f| {
        let entry = f.unwrap();
        let filename = entry.file_name();

        if entry.file_type().unwrap().is_dir() {
            recurse_zip_file(zip, &path.join(&filename), &relative_path.join(&filename));
        } else {
            zip.start_file_from_path(relative_path.join(&filename), SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&fs::read(path.join(&filename)).unwrap())
                .unwrap();
        }
    });
}

pub fn watch_folder(path: PathBuf) {
    let cloned = path.clone();

    let mut debouncer = new_debouncer(Duration::from_secs(1), None, move |result| match result {
        Ok(_) => file_update_callback(&cloned),
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
