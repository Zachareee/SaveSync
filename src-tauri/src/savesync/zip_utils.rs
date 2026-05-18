use std::{
    cmp::max,
    fs,
    io::{Cursor, Write},
    path::Path,
    time::SystemTime,
};

use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

use crate::savesync::fs_utils::recurse_directories;

pub fn extract(directory: impl AsRef<Path>, buffer: Vec<u8>) {
    ZipArchive::new(Cursor::new(buffer))
        .unwrap()
        .extract(directory)
        .unwrap();
}

pub fn zip_dir<P>(path: P) -> (Vec<u8>, SystemTime)
where
    P: AsRef<Path>,
{
    let buffer = Cursor::new(vec![]);
    let mut zip = ZipWriter::new(buffer);

    let date = recurse_directories(
        path.as_ref(),
        SystemTime::UNIX_EPOCH,
        &mut |p, relative_path, e| {
            let filename = e.file_name();
            zip.start_file_from_path(relative_path.join(&filename), SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&fs::read(p.join(&filename)).unwrap())
                .unwrap();
            e.metadata()?.modified()
        },
        &max,
    )
    .unwrap_or(SystemTime::UNIX_EPOCH);
    (
        zip.finish_into_readable()
            .unwrap()
            .into_inner()
            .into_inner(),
        date,
    )
}

