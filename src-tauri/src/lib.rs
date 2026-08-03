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
use tauri::{
    menu::{MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, RunEvent, WindowEvent,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_notification::NotificationExt;

static APP_INSTANCE: OnceLock<AppHandle> = OnceLock::new();
static APP_STORE: OnceLock<Arc<AppStore>> = OnceLock::new();

pub struct AppState {
    pub tags: HashSet<String>,
    pub buffers: HashMap<(String, OsString), Vec<u8>>,
    pub plugin: Option<Plugin>,
    pub server_port: Option<u16>,
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
            server_port: None,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
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
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_window" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => (),
        })
        .setup(|app| {
            emit_listeners(app);
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(
                    &MenuBuilder::new(app)
                        .items(&[&MenuItem::with_id(
                            app,
                            "show_window",
                            "Show Window",
                            true,
                            None::<Box<str>>,
                        )
                        .unwrap()])
                        .quit()
                        .build()?,
                )
                .build(app)?;

            #[cfg(desktop)]
            {
                app.handle()
                    .plugin(tauri_plugin_single_instance::init(|app, _, _| {
                        app.get_webview_window("main")
                            .expect("No main window found")
                            .set_focus()
                            .expect("Unable to focus main window");
                    }))?;

                app.handle().plugin(tauri_plugin_autostart::init(
                    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                    None,
                ))?;

                app.deep_link().on_open_url(|e| {
                    println!("Urls: {:?}", e.urls());
                });
            }

            let _ = APP_STORE.set(Arc::new(AppStore::new(app)));
            let _ = APP_INSTANCE.set(app.app_handle().to_owned());
            Ok(())
        })
        .manage(RwLock::new(AppState::default()))
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if app_store().close_behaviour() {
                    api.prevent_close();
                    window.hide().unwrap();
                    app_handle()
                        .notification()
                        .builder()
                        .title("savesync")
                        .body("The app has been minimised to the tray")
                        .sound("Default")
                        .show()
                        .unwrap();
                }
            }
            _ => (),
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
