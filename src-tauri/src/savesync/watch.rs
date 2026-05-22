use notify_debouncer_full::{new_debouncer, notify::*, Debouncer, RecommendedCache};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::Duration,
};

use crate::{app_store, read_app_state, savesync::zip_utils};

use super::zip_utils::zip_dir;

static WATCHERS: LazyLock<
    Mutex<HashMap<(String, OsString), Debouncer<RecommendedWatcher, RecommendedCache>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

const ZIPEXTENSION: &'static str = "savesynczip";

pub fn upload_file(tag: &str, path: &PathType) {
    let (zipbuffer, date) = match path {
        PathType::Directory(path) => zip_dir(&app_store().get_mapping(tag).unwrap().join(&path)),
        PathType::File(path) => {
            let abspath = &app_store().get_mapping(tag).unwrap().join(&path);
            (
                fs::read(abspath).unwrap(),
                fs::metadata(abspath).unwrap().modified().unwrap(),
            )
        }
    };

    let refpath = match path {
        PathType::Directory(path_buf) => path_buf.with_extension(ZIPEXTENSION),
        PathType::File(path_buf) => path_buf.to_owned(),
    };

    read_app_state(|s| {
        s.plugin_ref()
            .upload(
                tag.as_bytes(),
                refpath.as_os_str().as_encoded_bytes(),
                date,
                zipbuffer.as_slice(),
            )
            .unwrap()
    });
}

pub fn handle_buffer(path: impl AsRef<Path>, foldername: &PathType, buffer: Vec<u8>) {
    if let PathType::File(foldername) = foldername {
        fs::write(path.as_ref().join(foldername), buffer).unwrap();
    } else {
        zip_utils::extract(path.as_ref().with_extension(""), buffer).unwrap()
    }
}

fn setup_watcher(key: (String, OsString)) -> Debouncer<RecommendedWatcher, RecommendedCache> {
    let (tag, path) = key.clone();
    let path = strip_zip_extension(path);

    let mut debouncer = new_debouncer(Duration::from_secs(1), None, move |result| match result {
        Ok(_) => upload_file(&tag, &path),
        Err(err) => println!("{err:?}"),
    })
    .unwrap();

    let (tag, path) = key.clone();

    debouncer
        .watch(
            &app_store().get_mapping(&tag).unwrap().join(path),
            RecursiveMode::Recursive,
        )
        .unwrap();

    debouncer
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum PathType {
    Directory(PathBuf),
    File(PathBuf),
}

impl PathType {
    pub fn value(&self) -> &PathBuf {
        match self {
            PathType::Directory(path_buf) => path_buf,
            PathType::File(path_buf) => path_buf,
        }
    }
}

pub fn strip_zip_extension(path: impl AsRef<Path>) -> PathType {
    if path.as_ref().extension() == Some(&OsString::from(ZIPEXTENSION)) {
        PathType::Directory(path.as_ref().with_extension(""))
    } else {
        PathType::File(path.as_ref().to_path_buf())
    }
}

pub fn watch_folder(tag: &str, path: impl AsRef<Path>) {
    mutate_watchers(|map| {
        let key = (tag.to_owned(), path.as_ref().to_path_buf().into_os_string());
        if !map.contains_key(&key) {
            map.insert(key.clone(), setup_watcher(key));
        }
    })
}

pub fn toggle_watch(tag: &str, path: &OsString) -> bool {
    mutate_watchers(|map| {
        let key = (tag.to_owned(), path.to_owned());

        // !exist, !initial => add
        // !exist, initial => add
        // exist, !initial => remove
        // exist, initial => nothing

        let path = Path::new(path);
        let abspath = app_store().get_mapping(&tag).unwrap().join(&path);
        let pathbuf = if abspath.is_dir() {
            path.with_extension(ZIPEXTENSION)
        } else {
            path.into()
        };
        match map.contains_key(&key) {
            true => {
                map.remove(&key);
                read_app_state(|s| {
                    s.plugin_ref()
                        .remove(tag.as_bytes(), pathbuf.as_os_str().as_encoded_bytes())
                        .unwrap()
                });
                false
            }
            false => {
                map.insert(key.clone(), setup_watcher(key));
                true
            }
        }
    })
}

pub fn watched_folders() -> Vec<(String, OsString)> {
    mutate_watchers(|map| {
        map.iter()
            .map(|((tag, path), _)| (tag.into(), path.into()))
            .collect()
    })
}

pub fn dump_watchers() {
    mutate_watchers(|map| map.clear());
}

pub fn drop_watchers(watchers: Vec<(String, OsString)>) {
    mutate_watchers(|map| {
        watchers.iter().for_each(|k| {
            map.remove(k);
        })
    });
}

pub fn mutate_watchers<F, T>(func: F) -> T
where
    F: FnOnce(
        &mut HashMap<(String, OsString), Debouncer<RecommendedWatcher, RecommendedCache>>,
    ) -> T,
{
    func(&mut WATCHERS.lock().unwrap())
}
