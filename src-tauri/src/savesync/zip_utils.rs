use std::{
    fs,
    io::{Cursor, Seek, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

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

    let date = recurse_zip_file(&mut zip, path, &PathBuf::new());
    (
        zip.finish_into_readable()
            .unwrap()
            .into_inner()
            .into_inner(),
        date,
    )
}

fn recurse_zip_file<P>(
    zip: &mut ZipWriter<P>,
    path: impl AsRef<Path>,
    relative_path: impl AsRef<Path>,
) -> SystemTime
where
    P: Write + Seek,
{
    let path = path.as_ref();
    let relative_path = relative_path.as_ref();

    fs::read_dir(path)
        .unwrap()
        .into_iter()
        .fold(SystemTime::UNIX_EPOCH, |accum, f| {
            let entry = f.unwrap();
            let filename = entry.file_name();

            let date = if entry.file_type().unwrap().is_dir() {
                recurse_zip_file(zip, &path.join(&filename), &relative_path.join(&filename))
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
