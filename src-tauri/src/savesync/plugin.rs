use mlua::{BString, FromLuaMulti, Function, IntoLuaMulti, Lua, LuaOptions, LuaSerdeExt, StdLib};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::Path,
    sync::{Arc, LazyLock},
    time::SystemTime,
};

use super::config_paths;

const FIELD_MATCHER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`(.*)`").expect("Unable to compile regex"));

#[derive(Debug)]
pub struct Plugin {
    backend: Lua,
    filename: Arc<OsString>,
}

/// Gets file's last modified date
/// Plugin developers can optionally attach the
/// file buffer to reduce API calls where possible
pub struct FileDetails {
    pub tag: String,
    pub folder_name: OsString,
    pub last_modified: SystemTime,
    pub data: Option<Vec<u8>>,
}

#[derive(Deserialize)]
struct InterFileDetails {
    pub tag: String,
    pub folder_name: String,
    pub last_modified: SystemTime,
    pub data: Option<BString>,
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
            folder_name: folder_name.into(),
            last_modified,
            data: data.map(|s| s.into()),
        }
    }
}

impl Plugin {
    pub fn filename(&self) -> Arc<OsString> {
        self.filename.clone()
    }

    pub fn new(servicename: &OsStr) -> PluginResult<Plugin> {
        let backend = unsafe { Lua::unsafe_new_with(StdLib::ALL_SAFE, LuaOptions::new()) };

        let servicepath = config_paths::plugin().join(servicename);
        let path = include_path(&servicepath, "lua");
        let cpath = include_path(&servicepath, "dll");


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
            .call::<()>(servicepath.join("main.lua").as_path())
            .map_err(|e| format!("Error parsing {}: {e}", servicepath.to_string_lossy()))?;


        Ok(Plugin {
            backend,
            filename: Arc::new(servicename.to_owned()),
        })
    }

    pub fn info(&self) -> PluginResult<PluginInfo> {
        self.backend
            .from_value(mlua::Value::Table(self.run_function("Info", ())?))
            .map_err(|e| {
                FIELD_MATCHER
                    .captures(&e.to_string())
                    .map(|cap| format!("Plugin {} was not found", cap.extract::<1>().1[0]))
                    .unwrap()
            })
            .map(
                |InterPluginInfo {
                     name,
                     description,
                     author,
                     icon_url,
                 }| PluginInfo {
                    filename: self.filename(),
                    name,
                    description,
                    author,
                    icon_url,
                },
            )
    }

    fn read_creds(&self) -> Option<String> {
        let mut filename = self.filename.to_os_string();
        filename.push(".auth");

        fs::read_to_string(super::config_paths::creds().join(&filename)).ok()
    }

    fn write_creds(&self, credentials: &str) -> std::io::Result<()> {
        fs::write(self.filename.to_os_string(), credentials)
    }

    pub fn validate(&self, redirect_uri: &str) -> PluginResult<Option<String>> {
        self.run_function("Validate", (self.read_creds(), redirect_uri))
    }

    pub fn process_save_credentials(&self, url: &str) -> PluginResult<String> {
        self.run_function("Extract_credentials", url)
            .inspect(|s: &String| {
                let _ = self.write_creds(s);
            })
    }

    pub fn abort(&self) -> PluginResult<()> {
        self.run_function("Abort", ())
            .and_then(|msg: Option<String>| match msg {
                Some(msg) => Err(msg),
                None => Ok(()),
            })
    }

    pub fn upload(
        &self,
        tag: &str,
        folder_name: &OsStr,
        date: SystemTime,
        buffer: mlua::BString,
    ) -> PluginResult<()> {
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
    }

    pub fn download(&self, tag: &str, folder_name: &OsStr) -> PluginResult<Vec<u8>> {
        println!("Download called");
        self.run_function::<mlua::BString>("Download", (tag, folder_name))
            .map(Into::into)
    }

    pub fn remove(&self, tag: &str, folder_name: &OsStr) -> PluginResult<()> {
        self.run_function("Remove", (tag, folder_name))
    }

    pub fn read_cloud(&self) -> PluginResult<Vec<FileDetails>> {
        println!("Read_cloud called");
        self.run_function::<Vec<_>>("Read_cloud", ())?
            .into_iter()
            .map(|table| {
                self.backend
                    .from_value::<InterFileDetails>(mlua::Value::Table(table))
                    .map(Into::into)
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    fn run_function<T>(&self, fn_name: &str, args: impl IntoLuaMulti) -> PluginResult<T>
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

#[derive(Deserialize)]
struct InterPluginInfo {
    name: String,
    description: String,
    author: String,
    icon_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginInfo {
    name: String,
    description: String,
    author: String,
    icon_url: Option<String>,
    filename: Arc<OsString>,
}

fn include_path<T>(servicename: T, ext: &str) -> String
where
    T: AsRef<Path>,
{
    servicename
        .as_ref()
        .join(["?.", ext].join(""))
        .to_string_lossy()
        .replace("\\", "/")
}

pub type PluginResult<T> = Result<T, String>;
