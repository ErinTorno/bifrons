use ::std::sync::Mutex;

use bevy::prelude::info;
use bevy_mod_scripting::{prelude::*, lua::api::bevy::LuaWorld};
use mlua::Lua;

use crate::system::level::LoadedLevel;

#[derive(Default)]
pub struct LevelAPIProvider;

impl APIProvider for LevelAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_level_lua(ctx).map_err(ScriptError::new_other)?;
        info!("finished attaching LevelAPIProvider");
        Ok(())
    }
}

fn attach_level_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    let table = ctx.create_table()?;
    table.set("name", ctx.create_function(|ctx, ()| {
            match ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_resource::<LoadedLevel>() {
                Some(ll) => Ok(Some(ll.level.name.to_string())),
                None     => Ok(None)
            }
        })?
    )?;
    ctx.globals().set("Level", table)?;
    Ok(())
}