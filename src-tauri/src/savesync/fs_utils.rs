use std::{
    fs::{read_dir, DirEntry, FileType},
    path::Path,
};

pub trait FolderItems {
    fn get_folders(&self) -> Result<Vec<DirEntry>, std::io::Error>;
    fn get_files(&self) -> Result<Vec<DirEntry>, std::io::Error>;
}

impl FolderItems for Path {
    fn get_folders(&self) -> Result<Vec<DirEntry>, std::io::Error> {
        iter_dir_entries(self, FileType::is_dir)
    }

    fn get_files(&self) -> Result<Vec<DirEntry>, std::io::Error> {
        iter_dir_entries(self, FileType::is_file)
    }
}

fn iter_dir_entries<F>(path: &Path, filter: F) -> Result<Vec<DirEntry>, std::io::Error>
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
