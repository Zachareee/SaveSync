use std::{
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::SystemTime,
};

use mlua::{FromLuaMulti, Function, IntoLuaMulti, Lua, LuaOptions, LuaSerdeExt, StdLib};

use regex::Regex;
use serde::Deserialize;

const FIELD_MATCHER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`(.*)`").expect("Unable to compile regex"));

#[derive(Debug)]
pub struct Plugin {
    backend: Lua,
    filename: Arc<OsString>,
}

/// Gets file's last modified date
/// Plugin developers can optionally attach
/// the file buffer to reduce API calls
pub struct FileDetails {
    pub tag: String,
    pub folder_name: String,
    pub last_modified: SystemTime,
    pub data: Option<Vec<u8>>,
}

#[derive(Deserialize)]
struct InterFileDetails {
    pub tag: String,
    pub folder_name: String,
    pub last_modified: SystemTime,
    pub data: Option<bytes::Bytes>,
}

impl From<InterFileDetails> for FileDetails {
    fn from(
        InterFileDetails {
            tag,
            folder_name,
            last_modified,
            data,
        }: InterFileDetails,
    ) -> Self {
        Self {
            tag,
            folder_name,
            last_modified,
            data: data.map(|s| s.into()),
        }
    }
}

impl Plugin {
    pub fn filename(&self) -> Arc<OsString> {
        self.filename.clone()
    }

    pub fn info(&self) -> Result<PluginInfo, String> {
        self.backend
            .from_value(mlua::Value::Table(self.run_function("Info", ())?))
            .map_or_else(
                |e| {
                    Err(FIELD_MATCHER
                        .captures(&e.to_string())
                        .map(|cap| format!("Plugin {} was not found", cap.extract::<1>().1[0]))
                        .unwrap())
                },
                |mut info: PluginInfo| {
                    info.filename = Some(self.filename());
                    Ok(info)
                },
            )
    }

    pub fn init(&self) -> Result<(), String> {
        let mut filename = self.filename.to_os_string();
        filename.push(".auth");

        let filename = super::config_paths::creds().join(&filename);

        self.run_function("Init", fs::read_to_string(&filename).ok())
            .and_then(
                |(creds, err): (Option<String>, Option<String>)| match creds {
                    Some(credentials) => {
                        fs::write(&filename, credentials).expect("Unable to write credentials");
                        Ok(())
                    }
                    None => Err(err.unwrap_or("Something went wrong".into())),
                },
            )
    }

    pub fn abort(&self) -> Result<(), String> {
        self.run_function("Abort", ())
            .and_then(|msg: Option<String>| match msg {
                Some(msg) => Err(msg),
                None => Ok(()),
            })
    }

    pub fn upload(
        &self,
        tag: &str,
        folder_name: OsString,
        date: SystemTime,
        buffer: mlua::BString,
    ) {
        self.run_function(
            "Upload",
            (
                tag.to_owned(),
                folder_name,
                date.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                buffer,
            ),
        )
        .unwrap()
    }

    pub fn download(&self, tag: &str, folder_name: &str) -> Vec<u8> {
        println!("Download called");
        self.run_function::<mlua::BString>("Download", (tag, folder_name))
            .unwrap()
            .into()
    }

    pub fn remove(&self, tag: &str, folder_name: &OsStr) {
        self.run_function::<()>("Remove", (tag, folder_name))
            .unwrap();
    }

    pub fn read_cloud(&self) -> Vec<FileDetails> {
        println!("Read_cloud called");
        self.run_function::<Vec<_>>("Read_cloud", ())
            .unwrap()
            .into_iter()
            .map(|table| {
                self.backend
                    .from_value::<InterFileDetails>(mlua::Value::Table(table))
                    .unwrap()
                    .into()
            })
            .collect()
    }

    fn run_function<T>(&self, fn_name: &str, args: impl IntoLuaMulti) -> Result<T, String>
    where
        T: FromLuaMulti,
    {
        self.backend
            .globals()
            .get::<Function>(fn_name)
            .map_err(|_| format!("{fn_name} function not defined"))?
            .call(args)
            .map_err(|e| format!("Error while calling {fn_name}: {e}"))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PluginInfo {
    name: String,
    description: String,
    author: String,
    icon_url: Option<String>,
    filename: Option<Arc<OsString>>,
}

pub fn load_plugin(servicename: &PathBuf) -> Result<Plugin, String> {
    // enable DLL loading
    let backend = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::new()) };

    let path = include_path(servicename, "lua");
    let cpath = include_path(servicename, "dll");

    backend
        .load(format!(
            "package.path = '{path};' .. package.path; package.cpath = '{cpath};' .. package.cpath"
        ))
        .exec()
        .map_err(|e| format!("Unable to change package path: {e}"))?;
    backend
        .globals()
        .get::<Function>("dofile")
        .unwrap() // dofile() should always be available in lua runtime
        .call::<()>(servicename.join("main.lua").as_path())
        .map_err(|e| format!("Error parsing {}: {e}", servicename.to_string_lossy()))?;

    Ok(Plugin {
        backend,
        filename: servicename.file_name().unwrap().to_os_string().into(),
    })
}

fn include_path(servicename: &PathBuf, ext: &str) -> String {
    servicename
        .join(["?.", ext].join(""))
        .to_string_lossy()
        .replace("\\", "/")
}
