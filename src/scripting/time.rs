use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data::lua::LuaWorld;

use super::{LuaMod};

#[derive(Clone, Copy, Debug, Default, Deserialize, Inspectable, PartialEq, Resource, Serialize, Reflect)]
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
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: LuaTime| Ok(this == &that));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("time{{delta = {}, elapsed = {}}}", this.delta, this.elapsed)));
    }
}
impl LuaMod for LuaTime {
    fn mod_name() -> &'static str { "Time" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("elapsed", lua.create_function(|ctx, ()| {
                let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
                let w     = world.read();
                let time  = w.resource::<Time>();
                Ok(time.elapsed_seconds_f64())
            })?
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Inspectable, PartialEq, Serialize)]
pub struct LuaTimer {
    #[serde(default)]
    pub start:    f64,
    pub duration: f64,
    pub repeat:   bool,
}
// impl LuaMod for LuaTimer {
//     fn mod_name() -> &'static str { "Timer" }
//     fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
//         Ok(())
//     }
// }