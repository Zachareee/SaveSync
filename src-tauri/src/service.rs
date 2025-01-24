#[derive(serde::Serialize, serde::Deserialize)]
pub struct Service {
    name: String,
    description: String,
    author: String,
}

pub fn plugin_info(servicename: &str) -> mlua::Result<Service> {
    let lua = mlua::Lua::new();

    let servicename = servicename.replace("\\", "/");

    lua.load(format!("dofile '{servicename}'")).eval::<()>()?;

    let table = lua.load("Info()").eval::<mlua::Table>()?;

    Ok(Service {
        name: table.get("name")?,
        description: table.get("description")?,
        author: table.get("author")?,
    })
}
