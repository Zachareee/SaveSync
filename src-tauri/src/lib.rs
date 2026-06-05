// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod listeners;
mod savesync;

use commands::{filetree, get_mapping, get_plugins, get_watched_folders, set_mapping};
use listeners::emit_listeners;
use savesync::{plugin::Plugin, store::AppStore};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    ops::Deref,
    sync::{Arc, OnceLock, RwLock},
};
use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_deep_link::DeepLinkExt;

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();
static APP_STORE: OnceLock<Arc<AppStore>> = OnceLock::new();

pub struct AppState {
    pub tags: HashSet<String>,
    pub buffers: HashMap<(String, OsString), Vec<u8>>,
    pub plugin: Option<Plugin>,
}

impl AppState {
    pub fn plugin_mut_ref(&mut self) -> &mut Plugin {
        self.plugin.as_mut().unwrap()
    }

    pub fn plugin_ref(&self) -> &Plugin {
        self.plugin.as_ref().unwrap()
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            tags: HashSet::new(),
            buffers: HashMap::new(),
            plugin: None,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_, _, _| {}));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_oauth::init())
        .invoke_handler(tauri::generate_handler![
            get_plugins,
            get_mapping,
            set_mapping,
            get_watched_folders,
            filetree
        ])
        .setup(|app| {
            emit_listeners(app);

            let _ = APP_STORE.set(Arc::new(AppStore::new(app)));

            app.manage(RwLock::new(AppState::default()));

            #[cfg(desktop)]
            app.deep_link().on_open_url(|e| {
                println!("Urls: {:?}", e.urls());
            });

            APP_INSTANCE.set(app.app_handle().to_owned()).unwrap();
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Error while building tauri application")
        .run(|_, event| match event {
            RunEvent::ExitRequested { .. } => {
                app_store().save().unwrap();
            }
            _ => (),
        })
}

pub fn app_handle() -> AppHandle {
    APP_INSTANCE.get().unwrap().to_owned()
}

pub fn app_store() -> Arc<AppStore> {
    APP_STORE.get().unwrap().clone()
}

pub fn write_app_state<F, T>(func: F) -> T
where
    F: FnOnce(&mut AppState) -> T,
{
    func(
        &mut APP_INSTANCE
            .get()
            .unwrap()
            .state::<RwLock<AppState>>()
            .write()
            .unwrap(),
    )
}

pub fn read_app_state<F, T>(func: F) -> T
where
    F: FnOnce(&AppState) -> T,
{
    func(
        APP_INSTANCE
            .get()
            .unwrap()
            .state::<RwLock<AppState>>()
            .read()
            .unwrap()
            .deref(),
    )
}
