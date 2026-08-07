use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};

use serde_json::{from_value, json, to_value, Map, Value};
use tauri::{Manager, Wry};
use tauri_plugin_store::{Result, Store, StoreBuilder};

pub struct AppStore {
    store: Arc<Store<Wry>>,
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
                .default("silenceMissingMappings", false)
                .default("hide_to_tray", true)
                .default("sync_notifications", true)
                .auto_save(Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    pub fn plugin(&self) -> Option<OsString> {
        from_value(self.store.get("plugin").unwrap_or_default()).ok()
    }

    pub fn set_plugin(&self, plugin: &OsStr) {
        self.store
            .set("plugin", to_value(plugin).unwrap_or_default());
    }

    pub fn path_mapping(&self) -> PathMapping {
        self.mapping()
            .into_iter()
            .map(|(k, v)| (k, from_value(v).unwrap_or_default()))
            .filter(|(_, v)| Path::new(v).exists())
            .collect()
    }

    pub fn set_mapping(&self, map: PathMapping) {
        self.store
            .set("path_mapping", to_value(map).unwrap_or_default())
    }

    fn mapping(&self) -> Map<String, Value> {
        self.store
            .get("path_mapping")
            .unwrap_or_default()
            .as_object()
            .unwrap_or(&Map::new())
            .to_owned()
    }

    pub fn set_last_sync(&self, time: SystemTime) {
        self.store.set(
            "last_sync",
            time.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    pub fn last_sync(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH
            + Duration::from_secs(
                self.store
                    .get("last_sync")
                    .unwrap_or_default()
                    .as_u64()
                    .unwrap_or_default(),
            )
    }

    pub fn get_mapping(&self, key: &str) -> Option<PathBuf> {
        self.path_mapping().get(key).map(|s| s.into())
    }

    pub fn resolve_path(&self, tag: &str, path: impl AsRef<Path>) -> PathBuf {
        self.get_mapping(tag).unwrap().join(&path)
    }

    pub fn close_behaviour(&self) -> bool {
        self.store
            .get("hide_to_tray")
            .as_ref()
            .and_then(Value::as_bool)
            .unwrap_or_default()
    }

    pub fn sync_notifications(&self) -> bool {
        self.store
            .get("sync_notifications")
            .as_ref()
            .and_then(Value::as_bool)
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        self.set_last_sync(SystemTime::now());
        self.set_mapping(self.path_mapping());
        self.store.save()
    }
}

pub type PathMapping = HashMap<String, OsString>;
