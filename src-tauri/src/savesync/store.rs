use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};

use serde_json::{Map, Value, from_value, json, to_value};
use tauri::{Manager, Wry};
use tauri_plugin_store::{Result, Store, StoreBuilder};

use crate::commands::env_resolve;

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

    pub fn plugin(&self) -> Option<OsString> {
        from_value(self.store.get("plugin").unwrap_or_default()).ok()
    }

    pub fn path_mapping(&self) -> PathMapping {
        self.store
            .get("path_mapping")
            .unwrap_or_default()
            .as_object()
            .unwrap_or(&Map::new())
            .to_owned()
            .into_iter()
            .map(|(k, v)| (k, from_value(v).unwrap_or_default()))
            .collect()
    }

    pub fn set_plugin(&self, plugin: &OsStr) {
        self.store.set("plugin", to_value(plugin).unwrap_or_default());
    }

    pub fn save(&self) -> Result<()> {
        self.store.set(
            "last_sync",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self.store.save()
    }

    pub fn set_mapping(&self, map: PathMapping) {
        self.store.set("path_mapping", to_value(map).unwrap_or_default())
    }

    fn mapping(&self) -> Value {
        self.store.get("path_mapping").unwrap_or_default()
    }

    pub fn last_sync(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH
            + Duration::from_secs(self.store.get("last_sync").unwrap_or_default().as_u64().unwrap_or_default())
    }

    pub fn get_mapping(&self, key: &str) -> Option<PathBuf> {
        self.mapping()
            .as_object()
            .unwrap_or(&Map::new())
            .get(key)
            .cloned()
            .map(|s| {
                let (envvar, folder): (String, OsString) = from_value(s).unwrap_or_default();
                env_resolve(&envvar).expect("Environment variable not found").join(folder).into()
            })
    }
}

pub type PathMapping = HashMap<String, (String, OsString)>;
