use bevy::prelude::*;
use mlua::prelude::*;

use crate::{scripting::LuaMod, data::lua::LuaWorld};

use super::handle::LuaHandle;

#[derive(Default)]
pub struct ImageAPI;
impl LuaMod for ImageAPI {
    fn mod_name() -> &'static str { "Image" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("load", lua.create_function(|lua, path: String| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let asset_server = w.resource::<AssetServer>();
            Ok(LuaHandle::from(asset_server.load::<Image, _>(&path)))
        })?)?;
        Ok(())
    }
}