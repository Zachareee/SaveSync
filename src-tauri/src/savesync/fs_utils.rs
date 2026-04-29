use std::{
    fs::{read_dir, DirEntry, FileType},
    path::{Path, PathBuf},
};

use crate::app_store;

/// fs_utils internal representation of a Result<Vec<DirEntry>, std::io::Error>
type DirResult = Result<Vec<DirEntry>, std::io::Error>;

pub trait FolderItems {
    fn get_folders(&self) -> DirResult;
    //fn get_files(&self) -> DirResult;
}

impl FolderItems for Path {
    fn get_folders(&self) -> DirResult {
        iter_dir_entries(self, FileType::is_dir)
    }
    //
    //fn get_files(&self) -> DirResult {
    //    iter_dir_entries(self, FileType::is_file)
    //}
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
