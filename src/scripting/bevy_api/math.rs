use bevy::prelude::*;
use mlua::prelude::*;

use crate::scripting::LuaMod;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LuaVec2(pub Vec2);
impl LuaVec2 {
    pub fn new(v: Vec2) -> Self { LuaVec2(v) }
}
impl From<Vec2> for LuaVec2 {
    fn from(v: Vec2) -> Self { LuaVec2(v) }
}
impl From<LuaVec2> for Vec2 {
    fn from(LuaVec2(v): LuaVec2) -> Self { v }
}
impl LuaUserData for LuaVec2 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.0.x));
        fields.add_field_method_set("x", |_, this, x| Ok(this.0.x = x));
        fields.add_field_method_get("y", |_, this| Ok(this.0.y));
        fields.add_field_method_set("y", |_, this, y| Ok(this.0.y = y));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, that: LuaVec2| Ok(LuaVec2(this.0 + that.0)));
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, that: LuaVec2| Ok(LuaVec2(this.0 / that.0)));
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, that: LuaVec2| Ok(LuaVec2(this.0 + that.0)));
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, that: LuaVec2| Ok(LuaVec2(this.0 * that.0)));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{{x={}, y={}}}", this.0.x, this.0.y)));
        methods.add_method("clone", |_, this, ()| Ok(this.clone()));
    }
}
impl LuaMod for LuaVec2 {
    fn mod_name() -> &'static str { "Vec2" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("new", lua.create_function(|_, (x, y)| Ok(LuaVec2::new(Vec2::new(x, y))))?)?;
        table.set("one", lua.create_function(|_, ()| Ok(LuaVec2::new(Vec2::ONE)))?)?;
        table.set("splat", lua.create_function(|_, f| Ok(LuaVec2::new(Vec2::splat(f))))?)?;
        table.set("zero", lua.create_function(|_, ()| Ok(LuaVec2::new(Vec2::ZERO)))?)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LuaVec3(pub Vec3);
impl LuaVec3 {
    pub fn new(v: Vec3) -> Self { LuaVec3(v) }
}
impl From<Vec3> for LuaVec3 {
    fn from(v: Vec3) -> Self { LuaVec3(v) }
}
impl From<LuaVec3> for Vec3 {
    fn from(LuaVec3(v): LuaVec3) -> Self { v }
}
impl LuaUserData for LuaVec3 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.0.x));
        fields.add_field_method_set("x", |_, this, x| Ok(this.0.x = x));
        fields.add_field_method_get("y", |_, this| Ok(this.0.y));
        fields.add_field_method_set("y", |_, this, y| Ok(this.0.y = y));
        fields.add_field_method_get("z", |_, this| Ok(this.0.z));
        fields.add_field_method_set("z", |_, this, z| Ok(this.0.z = z));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, that: LuaVec3| Ok(LuaVec3(this.0 + that.0)));
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, that: LuaVec3| Ok(LuaVec3(this.0 / that.0)));
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, that: LuaVec3| Ok(LuaVec3(this.0 + that.0)));
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, that: LuaVec3| Ok(LuaVec3(this.0 * that.0)));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{{x={}, y={}, z={}}}", this.0.x, this.0.y, this.0.z)));
        methods.add_method("clone", |_, this, ()| Ok(this.clone()));
    }
}
impl LuaMod for LuaVec3 {
    fn mod_name() -> &'static str { "Vec3" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("new", lua.create_function(|_, (x, y, z)| Ok(LuaVec3::new(Vec3::new(x, y, z))))?)?;
        table.set("one", lua.create_function(|_, ()| Ok(LuaVec3::new(Vec3::ONE)))?)?;
        table.set("splat", lua.create_function(|_, f| Ok(LuaVec3::new(Vec3::splat(f))))?)?;
        table.set("zero", lua.create_function(|_, ()| Ok(LuaVec3::new(Vec3::ZERO)))?)?;
        Ok(())
    }
}