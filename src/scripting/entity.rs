use ::std::sync::Mutex;

use bevy::prelude::{info, warn, error, Vec3, Visibility};
use bevy_mod_scripting::{prelude::*, lua::api::bevy::{LuaEntity, LuaWorld}};
use mlua::Lua;
use serde::{Deserialize, Serialize};

use super::{init_luamod, LuaMod};

#[derive(Default)]
pub struct EntityAPIProvider;

impl APIProvider for EntityAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_entity_lua(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

fn attach_entity_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    let table = ctx.create_table()?;
    table.set("set_visible", ctx.create_function(|ctx, (entity, is_visible): (LuaEntity, bool)| {
            if let Some(mut ent_mut) = ctx.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity_mut(entity.inner()?) {
                ent_mut.insert(Visibility { is_visible });
            }
            Ok(())
        })?
    )?;
    ctx.globals().set("Entity", table)?;
    Ok(())
}