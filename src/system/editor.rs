use bevy::prelude::Component;
use bevy_mod_scripting::lua::api::bevy::{LuaEntity, LuaWorld};
use derive_more::Display;
use mlua::prelude::*;
use serde::{Serialize, Deserialize};

use crate::scripting::LuaMod;

#[derive(Clone, Component, Debug, Deserialize, Display, Eq, PartialEq, Serialize)]
pub struct AssetInfo {
    pub path: String,
}
impl AssetInfo {
    pub fn new<S>(s: S) -> Self where S: AsRef<str> {
        AssetInfo {
            path: s.as_ref().trim().to_string(),
        }
    }
}
impl LuaMod for AssetInfo {
    fn mod_name() -> &'static str { "Asset" }

    fn register_defs(ctx: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("path_of", ctx.create_function(|lua, entity: LuaEntity| {
            let entity = entity.inner()?;
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            if let Some(ent_ref) = w.get_entity(entity) {
                Ok(ent_ref.get().cloned().map(|a: AssetInfo| a.path))
            } else { Ok(None) }
        })?)?;
        Ok(())
    }
}