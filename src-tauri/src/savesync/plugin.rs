use std::{path::PathBuf, sync::LazyLock};

use mlua::{Function, Lua, LuaSerdeExt, Table};

use regex::Regex;

const FIELD_MATCHER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`(.*)`").expect("Unable to compile regex"));

pub struct Plugin {
    backend: Lua,
}

impl Plugin {
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
            .map_err(|e| {
                FIELD_MATCHER
                    .captures(&e.to_string())
                    .map(|cap| format!("Plugin {} was not found", cap.extract::<1>().1[0]))
                    .unwrap()
            })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PluginInfo {
    name: String,
    description: String,
    author: String,
}

pub fn load_plugin(servicename: &PathBuf) -> Result<Plugin, String> {
    let backend = Lua::new();

    backend
        .globals()
        .get::<Function>("dofile")
        .unwrap() // dofile() should always be available in lua runtime
        .call::<()>(servicename.as_path())
        .map_err(|e| format!("Error parsing {}: {e}", servicename.to_string_lossy()))?;

    Ok(Plugin { backend })
}
