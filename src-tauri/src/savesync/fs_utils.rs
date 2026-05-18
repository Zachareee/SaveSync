use std::{
    fs::{read_dir, DirEntry, FileType},
    io,
    path::{Path, PathBuf},
};

use crate::app_store;

/// fs_utils internal representation of a Result<Vec<DirEntry>, std::io::Error>
type DirResult = Result<Vec<DirEntry>, std::io::Error>;

pub trait FolderItems {
    fn get_folders(&self) -> DirResult;
    fn get_files(&self) -> DirResult;
}

impl FolderItems for Path {
    fn get_folders(&self) -> DirResult {
        iter_dir_entries(self, FileType::is_dir)
    }

    fn get_files(&self) -> DirResult {
        iter_dir_entries(self, FileType::is_file)
    }
}

fn iter_dir_entries<F>(path: &Path, filter: F) -> DirResult
where
    F: Fn(&FileType) -> bool,
{
    read_dir(path)?
        .filter(|p| {
            p.as_ref()
                .is_ok_and(|entry| entry.file_type().as_ref().is_ok_and(&filter))
        })
        .collect()
}

pub fn resolve_path<P>(tag: &str, path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    app_store().get_mapping(&tag).unwrap().join(path)
}

pub fn recurse_directories<V, F, A>(
    path: &Path,
    dvalue: V,
    lambda: &mut F,
    accumulator: &A,
) -> std::io::Result<V>
where
    F: FnMut(&Path, &PathBuf, DirEntry) -> io::Result<V>,
    A: Fn(V, V) -> V,
    V: Clone + Copy,
{
    let relative_path = PathBuf::new();
    recurse_directories_sub(path, &relative_path, dvalue, lambda, accumulator)
}

pub fn recurse_directories_sub<V, F, A>(
    path: &Path,
    relative_path: &PathBuf,
    dvalue: V,
    lambda: &mut F,
    accumulator: &A,
) -> std::io::Result<V>
where
    F: FnMut(&Path, &PathBuf, DirEntry) -> io::Result<V>,
    A: Fn(V, V) -> V,
    V: Clone + Copy,
{
    read_dir(path)?.try_fold(dvalue, |accum, entry| {
        let entry = entry?;
        let value = if entry.file_type()?.is_dir() {
            recurse_directories_sub(
                &path.join(entry.file_name()),
                &relative_path.join(entry.file_name()),
                dvalue,
                lambda,
                accumulator,
            )
        } else {
            lambda(path, relative_path, entry)
        };

        Ok(accumulator(accum, value?))
    })
}
