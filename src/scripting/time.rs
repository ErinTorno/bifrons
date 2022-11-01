use ::std::sync::Mutex;

use bevy::{prelude::{info}, time::Time};
use bevy_mod_scripting::{prelude::*, lua::api::bevy::LuaWorld};
use mlua::Lua;

#[derive(Default)]
pub struct TimeAPIProvider;

impl APIProvider for TimeAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_time_lua(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

fn attach_time_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    let table = ctx.create_table()?;
    table.set("delta", ctx.create_function(|ctx, ()| {
            match ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_resource::<Time>() {
                Some(t) => Ok(Some(t.delta_seconds())),
                None => Ok(None)
            }
        })?
    )?;
    table.set("elapsed", ctx.create_function(|ctx, ()| {
            match ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_resource::<Time>() {
                Some(t) => Ok(Some(t.seconds_since_startup())),
                None => Ok(None)
            }
        })?
    )?;
    ctx.globals().set("Time", table)?;
    Ok(())
}