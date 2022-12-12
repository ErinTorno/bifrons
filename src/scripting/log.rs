use bevy::prelude::{info, warn, error};
use mlua::prelude::*;

use crate::scripting::format_lua;

use super::LuaMod;

#[derive(Default)]
pub struct LogAPI;
impl LuaMod for LogAPI {
    fn mod_name() -> &'static str { "Log" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("error", lua.create_function(|_lua, values: LuaMultiValue| {
            Ok(error!("{}", format_lua(values)?))
        })?)?;
        table.set("info", lua.create_function(|_lua, values: LuaMultiValue| {
            Ok(info!("{}", format_lua(values)?))
        })?)?;
        table.set("warn", lua.create_function(|_lua, values: LuaMultiValue| {
            Ok(warn!("{}", format_lua(values)?))
        })?)?;
        Ok(())
    }
}