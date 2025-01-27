use std::{collections::HashMap, path::PathBuf};

use mlua::{prelude::LuaError, ExternalError, Function, Lua, Result, Table};
use serde_json::Value;

pub struct Plugin {
    backend: Lua,
}

enum JsonValue {
    String,
    #[allow(dead_code)]
    I32,
}

type TableTypes = (&'static str, JsonValue);

const PLUGIN_INFO: [TableTypes; 3] = [
    ("name", JsonValue::String),
    ("description", JsonValue::String),
    ("author", JsonValue::String),
];

impl Plugin {
    pub fn info(&self) -> Result<HashMap<String, Value>> {
        let table = self
            .backend
            .globals()
            .get::<Function>("Info")
            .map_err(|_| "Info() function not defined".into_lua_err())?
            .call::<Table>(())?;

        table.convert(&PLUGIN_INFO, "Plugin {} was not found")
    }
}

trait TableToMap {
    fn convert(
        &self,
        map_requirements: &[TableTypes],
        error_str: &str,
    ) -> Result<HashMap<String, Value>>;
}

impl TableToMap for Table {
    fn convert(
        &self,
        map_requirements: &[TableTypes],
        error_str: &str,
    ) -> Result<HashMap<String, Value>> {
        map_requirements
            .iter()
            .map(|(name, value_type)| {
                Ok((
                    name.to_string(),
                    match value_type {
                        JsonValue::String => {
                            Value::String(self.get(name.to_string()).map_err(|_: LuaError| {
                                error_str.replace("{}", name).into_lua_err()
                            })?)
                        }
                        JsonValue::I32 => Value::Number(
                            self.get::<i32>(name.to_string())
                                .map_err(|_: LuaError| {
                                    error_str.replace("{}", name).into_lua_err()
                                })?
                                .into(),
                        ),
                    },
                ))
            })
            .collect()
    }
}

pub fn load_plugin(servicename: &PathBuf) -> Result<Plugin> {
    let backend = Lua::new();

    backend
        .globals()
        .get::<Function>("dofile")?
        .call::<()>(servicename.as_path())?;

    Ok(Plugin { backend })
}
