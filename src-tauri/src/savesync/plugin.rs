use std::{path::PathBuf, rc::Rc, sync::LazyLock};

use mlua::{Function, Lua, LuaSerdeExt, Table};

use regex::Regex;

const FIELD_MATCHER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"`(.*)`").expect("Unable to compile regex"));

#[derive(Debug)]
pub struct Plugin {
    backend: Lua,
    file_name: Rc<PathBuf>,
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
            .map_or_else(
                |e| {
                    Err(FIELD_MATCHER
                        .captures(&e.to_string())
                        .map(|cap| format!("Plugin {} was not found", cap.extract::<1>().1[0]))
                        .unwrap())
                },
                |mut info: PluginInfo| {
                    info.file_name = Some(self.file_name.clone());
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
    file_name: Option<Rc<PathBuf>>,
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
        file_name: servicename.clone().into(),
    })
}
