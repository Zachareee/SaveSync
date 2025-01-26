use std::path::PathBuf;

use mlua::{ExternalError, Function, Lua, Result, Table};

pub struct Plugin {
    backend: Lua,
}

impl Plugin {
    pub fn info(&self) -> Result<PluginInfo> {
        let table = self
            .backend
            .globals()
            .get::<Function>("Info")
            .map_err(|_| "Info() function not defined".into_lua_err())?
            .call::<Table>(())?;

        Ok(PluginInfo {
            name: table
                .get("name")
                .map_err(|_| "Plugin name was not found".into_lua_err())?,
            description: table
                .get("description")
                .map_err(|_| "Plugin description was not found".into_lua_err())?,
            author: table
                .get("author")
                .map_err(|_| "Plugin author was not found".into_lua_err())?,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PluginInfo {
    name: String,
    description: String,
    author: String,
}

pub fn load_plugin(servicename: &PathBuf) -> Result<Plugin> {
    let backend = Lua::new();

    backend
        .globals()
        .get::<Function>("dofile")?
        .call::<()>(servicename.as_path())?;

    Ok(Plugin { backend })
}
