use bevy::prelude::*;
use mlua::prelude::*;

use crate::{scripting::LuaMod, data::lua::Any2};

#[derive(Default)]
pub struct MathAPI;
impl LuaMod for MathAPI {
    fn mod_name() -> &'static str { "Math" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("clamp", lua.create_function(|_, (n, min, max): (f64, f64, f64)| {
            Ok(n.clamp(min, max))
        })?)?;
        table.set("finite_or", lua.create_function(|_lua, (n, def): (f32, f32)| {
            Ok(if n.is_finite() { n } else { def })
        })?)?;
        table.set("pi", std::f64::consts::PI)?;
        Ok(())
    }
}

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
        methods.add_meta_method(LuaMetaMethod::Add, |lua, this, that: Any2<LuaVec2, LuaVec3>| match that {
            Any2::A(that) => LuaVec2(this.0 + that.0).to_lua(lua),
            Any2::B(that) => LuaVec3(this.0.extend(0.) + that.0).to_lua(lua),
        });
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, that: Any2<f32, LuaVec2>| match that {
            Any2::A(that) => Ok(LuaVec2(this.0 / that)),
            Any2::B(that) => Ok(LuaVec2(this.0 / that.0)),
        });
        methods.add_meta_method(LuaMetaMethod::Sub, |lua, this, that: Any2<LuaVec2, LuaVec3>| match that {
            Any2::A(that) => LuaVec2(this.0 + that.0).to_lua(lua),
            Any2::B(that) => LuaVec3(this.0.extend(0.) - that.0).to_lua(lua),
        });
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, that: Any2<f32, LuaVec2>| match that {
            Any2::A(that) => Ok(LuaVec2(this.0 * that)),
            Any2::B(that) => Ok(LuaVec2(this.0 * that.0)),
        });
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: LuaVec2| Ok(this.0 == that.0));
        methods.add_meta_method(LuaMetaMethod::Len, |_, _, ()| Ok(2));
        methods.add_meta_method(LuaMetaMethod::Pairs, |lua, this, ()| {
            let function = lua.create_function(|lua, (this, k): (LuaVec2, Option<String>)| {
                match k {
                    Some(s) => match s.as_str() {
                        "?" => {
                            let mut multi = LuaMultiValue::new();
                            multi.push_front(this.0.x.to_lua(lua)?);
                            multi.push_front("x".to_lua(lua)?);
                            Ok(multi)
                        },
                        "x" => {
                            let mut multi = LuaMultiValue::new();
                            multi.push_front(this.0.y.to_lua(lua)?);
                            multi.push_front("y".to_lua(lua)?);
                            Ok(multi)
                        },
                        _ => ().to_lua_multi(lua),
                    },
                    _ => ().to_lua_multi(lua),
                }
            })?;
            let mut multi = LuaMultiValue::new();
            multi.push_front("?".to_lua(lua)?);
            multi.push_front(this.clone().to_lua(lua)?);
            multi.push_front(function.to_lua(lua)?);
            Ok(multi)
        });
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{{x={}, y={}}}", this.0.x, this.0.y)));
        methods.add_method("clone", |_, this, ()| Ok(this.clone()));
        methods.add_method("extend", |_, this, z: f32| Ok(LuaVec3::new(this.0.extend(z))));
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
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, that: Any2<LuaVec3, LuaVec2>| match that {
            Any2::A(that) => Ok(LuaVec3(this.0 + that.0)),
            Any2::B(that) => Ok(LuaVec3(this.0 + that.0.extend(0.))),
        });
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, that: Any2<f32, LuaVec3>| match that {
            Any2::A(that) => Ok(LuaVec3(this.0 / that)),
            Any2::B(that) => Ok(LuaVec3(this.0 / that.0)),
        });
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, that: Any2<LuaVec3, LuaVec2>| match that {
            Any2::A(that) => Ok(LuaVec3(this.0 + that.0)),
            Any2::B(that) => Ok(LuaVec3(this.0 - that.0.extend(0.))),
        });
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, that: Any2<f32, LuaVec3>| match that {
            Any2::A(that) => Ok(LuaVec3(this.0 * that)),
            Any2::B(that) => Ok(LuaVec3(this.0 * that.0)),
        });
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: LuaVec3| Ok(this.0 == that.0));
        methods.add_meta_method(LuaMetaMethod::Len, |_, _, ()| Ok(3));
        methods.add_meta_method(LuaMetaMethod::Pairs, |lua, this, ()| {
            let function = lua.create_function(|lua, (this, k): (LuaVec3, Option<String>)| {
                match k {
                    Some(s) => match s.as_str() {
                        "?" => {
                            let mut multi = LuaMultiValue::new();
                            multi.push_front(this.0.x.to_lua(lua)?);
                            multi.push_front("x".to_lua(lua)?);
                            Ok(multi)
                        },
                        "x" => {
                            let mut multi = LuaMultiValue::new();
                            multi.push_front(this.0.y.to_lua(lua)?);
                            multi.push_front("y".to_lua(lua)?);
                            Ok(multi)
                        },
                        "y" => {
                            let mut multi = LuaMultiValue::new();
                            multi.push_front(this.0.z.to_lua(lua)?);
                            multi.push_front("z".to_lua(lua)?);
                            Ok(multi)
                        },
                        _ => ().to_lua_multi(lua),
                    },
                    _ => ().to_lua_multi(lua),
                }
            })?;
            let mut multi = LuaMultiValue::new();
            multi.push_front("?".to_lua(lua)?);
            multi.push_front(this.clone().to_lua(lua)?);
            multi.push_front(function.to_lua(lua)?);
            Ok(multi)
        });
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