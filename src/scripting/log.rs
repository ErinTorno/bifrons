use ::std::sync::Mutex;

use bevy::prelude::{info, warn, error};
use bevy_mod_scripting::prelude::*;
use mlua::Lua;

use crate::scripting::format_lua;

#[derive(Default)]
pub struct LogAPIProvider;

impl APIProvider for LogAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_logging_lua(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

fn attach_logging_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    let table = ctx.create_table()?;
    table.set("error", ctx.create_function(|_ctx, values: LuaMultiValue| {
        error!("{}", format_lua(values)?);
        Ok(())
    })?)?;
    table.set("info", ctx.create_function(|_ctx, values: LuaMultiValue| {
        info!("{}", format_lua(values)?);
        Ok(())
    })?)?;
    table.set("warn", ctx.create_function(|_ctx, values: LuaMultiValue| {
        warn!("{}", format_lua(values)?);
        Ok(())
    })?)?;
    ctx.globals().set("Log", table)?;
    Ok(())
}