use notify_debouncer_full::{new_debouncer, notify::*};
use std::{
    fs,
    io::{Cursor, Seek, Write},
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{app_handle, app_state};

use super::plugin::load_plugin;

fn file_update_callback(tag: &str, path: &PathBuf) {
    let buffer = Cursor::new(vec![]);
    let mut zip = ZipWriter::new(buffer);

    let date = recurse_zip_file(&mut zip, path, &PathBuf::new());

    load_plugin(&app_state(&app_handle()).try_lock().unwrap().plugin.clone())
        .unwrap()
        .upload(
            tag,
            path.as_os_str().to_owned(),
            date,
            zip.finish_into_readable()
                .unwrap()
                .into_inner()
                .into_inner()
                .into(),
        );
}

fn recurse_zip_file<T>(
    zip: &mut ZipWriter<T>,
    path: &PathBuf,
    relative_path: &PathBuf,
) -> SystemTime
where
    T: Write + Seek,
{
    fs::read_dir(path)
        .unwrap()
        .into_iter()
        .fold(UNIX_EPOCH, |accum, f| {
            let entry = f.unwrap();
            let filename = entry.file_name();

            let date = if entry.file_type().unwrap().is_dir() {
                recurse_zip_file(zip, &path.join(&filename), &relative_path.join(&filename));
                UNIX_EPOCH
            } else {
                zip.start_file_from_path(
                    relative_path.join(&filename),
                    SimpleFileOptions::default(),
                )
                .unwrap();
                zip.write_all(&fs::read(path.join(&filename)).unwrap())
                    .unwrap();
                entry.metadata().unwrap().modified().unwrap()
            };

            if accum < date {
                date
            } else {
                accum
            }
        })
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
