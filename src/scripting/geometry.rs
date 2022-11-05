use ::std::sync::Mutex;

use bevy::prelude::{info, warn, error, Vec3};
use bevy_mod_scripting::prelude::*;
use mlua::Lua;
use serde::{Deserialize, Serialize};

use crate::{scripting::format_lua, data::geometry::{LightKind, Light, LightAnim}};

use super::{init_luamod, LuaMod};

#[derive(Default)]
pub struct GeometryAPIProvider;

impl APIProvider for GeometryAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        init_luamod::<Light>(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<LightAnim>(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<LightKind>(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<LuaVec3>(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct LuaVec3(pub Vec3);
impl LuaUserData for LuaVec3 {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.0.x));
        fields.add_field_method_set("x", |_, this, x: f32| {
            this.0.x = x;
            Ok(())
        });
        fields.add_field_method_get("y", |_, this| Ok(this.0.y));
        fields.add_field_method_set("y", |_, this, y: f32| {
            this.0.y = y;
            Ok(())
        });
        fields.add_field_method_get("z", |_, this| Ok(this.0.z));
        fields.add_field_method_set("z", |_, this, z: f32| {
            this.0.z = z;
            Ok(())
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, that: LuaVec3| Ok(LuaVec3(this.0 + that.0)));
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, that: f32| Ok(LuaVec3(this.0 * that)));
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, that: LuaVec3| Ok(LuaVec3(this.0 - that.0)));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{{x = {}, y = {}, z = {}}}", this.0.x, this.0.y, this.0.z)));
    }
}
impl LuaMod for LuaVec3 {
    fn mod_name() -> &'static str { "Vec3" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("new", lua.create_function(|_ctx, (x, y, z)| {
            Ok(LuaVec3(Vec3::new(x, y, z)))
        })?)?;
        table.set("x", lua.create_function(|_ctx, ()| {
            Ok(LuaVec3(Vec3::X))
        })?)?;
        table.set("y", lua.create_function(|_ctx, ()| {
            Ok(LuaVec3(Vec3::Y))
        })?)?;
        table.set("z", lua.create_function(|_ctx, ()| {
            Ok(LuaVec3(Vec3::Z))
        })?)?;
        table.set("zero", lua.create_function(|_ctx, ()| {
            Ok(LuaVec3(Vec3::ZERO))
        })?)?;
        Ok(())
    }
}