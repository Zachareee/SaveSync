use std::{ffi::OsString, path::PathBuf, rc::Rc, sync::LazyLock};

use mlua::{Function, Lua, LuaSerdeExt, Table};

use regex::Regex;

const FIELD_MATCHER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`(.*)`").expect("Unable to compile regex"));

#[derive(Debug)]
pub struct Plugin {
    backend: Lua,
    filename: Rc<OsString>,
}

impl Plugin {
    pub fn filename(&self) -> Rc<OsString> {
        self.filename.clone()
    }

    pub fn info(&self) -> Result<PluginInfo, String> {
        self.backend
            .from_value(mlua::Value::Table(
                self.backend
                    .globals()
                    .get::<Function>("Info")
                    .map_err(|_| "Info() function not defined")?
                    .call::<Table>(())
                    .map_err(|_| "Info() function must return a table")?,
            ))
            .map_or_else(
                |e| {
                    Err(FIELD_MATCHER
                        .captures(&e.to_string())
                        .map(|cap| format!("Plugin {} was not found", cap.extract::<1>().1[0]))
                        .unwrap())
                },
                |mut info: PluginInfo| {
                    info.filename = Some(self.filename.clone());
                    Ok(info)
                },
            )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PluginInfo {
    name: String,
    description: String,
    author: String,
    icon_url: Option<String>,
    filename: Option<Rc<OsString>>,
}

pub fn load_plugin(servicename: &PathBuf) -> Result<Plugin, String> {
    let backend = Lua::new();

    backend
        .globals()
        .get::<Function>("dofile")
        .unwrap() // dofile() should always be available in lua runtime
        .call::<()>(servicename.as_path())
        .map_err(|e| format!("Error parsing {}: {e}", servicename.to_string_lossy()))?;

    Ok(Plugin {
        backend,
        filename: servicename.file_name().unwrap().to_os_string().into(),
    })
}
