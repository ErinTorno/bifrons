use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::scripting::{LuaMod};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Setting {

}
impl LuaMod for Setting {
    fn mod_name() -> &'static str { "Setting" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {

        Ok(())
    }
}
impl LuaUserData for Setting {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
    }
}
