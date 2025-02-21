use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};

use serde_json::{json, Value};
use tauri::{Manager, Wry};
use tauri_plugin_store::{Result, Store, StoreBuilder};

pub struct AppStore {
    store: Arc<Store<Wry>>,
}

impl Clone for AppStore {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
        }
    }
}

impl AppStore {
    pub fn new<M>(app: &M) -> AppStore
    where
        M: Manager<Wry>,
    {
        AppStore {
            store: StoreBuilder::new(app, "store.json")
                .default("plugin", "")
                .default("path_mapping", json!({}))
                .default("last_sync", 0)
                .auto_save(Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    pub fn plugin(&self) -> Option<PathBuf> {
        self.store
            .get("plugin")
            .unwrap()
            .as_str()
            .map(|s| Path::new(s).to_owned())
            .filter(|p| p.exists())
    }

    pub fn path_mapping(&self) -> HashMap<String, (String, OsString)> {
        self.store
            .get("path_mapping")
            .unwrap()
            .as_object()
            .unwrap()
            .to_owned()
            .into_iter()
            .map(|(k, v)| (k, serde_json::from_value(v).unwrap()))
            .collect()
    }

    pub fn set_plugin(&self, plugin: impl AsRef<Path>) {
        self.store.set("plugin", plugin.as_ref().to_str().unwrap());
    }

    pub fn save(&self) -> Result<()> {
        self.store.set(
            "last_sync",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        self.store.save()
    }

    pub fn set_mapping(&self, map: PathMapping) {
        self.store
            .set("path_mapping", serde_json::to_value(map).unwrap())
    }

    fn mapping(&self) -> Value {
        self.store.get("path_mapping").unwrap()
    }

    pub fn last_sync(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH
            + Duration::from_secs(self.store.get("last_sync").unwrap().as_u64().unwrap())
    }

    pub fn get_mapping(&self, key: &str) -> Option<PathBuf> {
        self.mapping()
            .as_object()
            .unwrap()
            .get(key)
            .and_then(|s| Some(Path::new(s.as_str().unwrap()).into()))
    }
}

pub type PathMapping = HashMap<String, (String, OsString)>;
