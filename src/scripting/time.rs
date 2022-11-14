use ::std::sync::Mutex;

use bevy::{time::Time};
use bevy_inspector_egui::Inspectable;
use bevy_mod_scripting::{prelude::*, lua::api::bevy::LuaWorld};
use mlua::Lua;
use serde::{Deserialize, Serialize};

use super::{init_luamod, LuaMod};

#[derive(Default)]
pub struct TimeAPIProvider;

impl APIProvider for TimeAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        init_luamod::<LuaTime>(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Inspectable, PartialEq, Serialize)]
pub struct LuaTime {
    #[serde(default)]
    pub delta:   f64,
    pub elapsed: f64,
}
impl LuaUserData for LuaTime {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("delta",   |_, this| Ok(this.delta));
        fields.add_field_method_get("elapsed", |_, this| Ok(this.elapsed));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("#time{{delta = {}, elapsed = {}}}", this.delta, this.elapsed)));
    }
}
impl LuaMod for LuaTime {
    fn mod_name() -> &'static str { "Time" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("elapsed", lua.create_function(|ctx, ()| {
                match ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_resource::<Time>() {
                    Some(t) => Ok(Some(t.seconds_since_startup())),
                    None => Ok(None)
                }
            })?
        )?;
        Ok(())
    }
}