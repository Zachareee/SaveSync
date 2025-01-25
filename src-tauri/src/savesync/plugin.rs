use std::path::PathBuf;

use mlua::{Function, Lua, Result, Table};

pub struct Plugin {
    backend: Lua,
}

impl Plugin {
    pub fn info(&self) -> Result<PluginInfo> {
        let table = self
            .backend
            .globals()
            .get::<Function>("Info")?
            .call::<Table>(())?;

        Ok(PluginInfo {
            name: table.get("name")?,
            description: table.get("description")?,
            author: table.get("author")?,
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
