use notify_debouncer_full::{new_debouncer, notify::*};
use std::{
    fs,
    io::{Cursor, Read, Seek, Write},
    path::PathBuf,
    time::Duration,
};
use zip::ZipWriter;

use crate::{app_handle, app_state};

use super::plugin::load_plugin;

fn file_update(path: &PathBuf) {
    let buffer = Cursor::new(vec![]);
    let mut zip = ZipWriter::new(buffer);

    recurse_zip_file(&mut zip, path);

    load_plugin(&app_state(&app_handle()).lock().unwrap().plugin)
        .unwrap()
        .sync(
            &zip.finish_into_readable()
                .unwrap()
                .into_inner()
                .bytes()
                .filter_map(|r| r.ok())
                .collect::<Vec<u8>>(),
        );
}

fn recurse_zip_file<T>(zip: &mut ZipWriter<T>, path: &PathBuf)
where
    T: Write + Seek,
{
    fs::read_dir(path).unwrap().into_iter().for_each(|f| {
        f.unwrap();
    });
}

pub fn watch_folder(_mapping: String, path: PathBuf) {
    let cloned = path.clone();
    new_debouncer(Duration::from_secs(1), None, move |result| match result {
        Ok(_) => file_update(&cloned),
        Err(err) => println!("{err:?}"),
    })
    .unwrap()
    .watch(path, RecursiveMode::Recursive)
    .unwrap();
}
