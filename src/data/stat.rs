use std::{collections::HashMap, ops::{Add, Sub, Mul, Div}};

use bevy::prelude::*;
use bevy_mod_scripting::lua::api::bevy::{LuaEntity, LuaWorld};
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::scripting::LuaMod;

#[derive(Clone, Component, Debug, Default, Deserialize, Serialize)]
pub struct Attributes {
    #[serde(default)]
    pools: HashMap<String, Pool>,
    #[serde(default)]
    stats: HashMap<String, Stat>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stat {
    pub base:     f32,
    #[serde(default, rename = "mod")]
    pub modifier: f32,
}
impl Stat {
    pub fn total(&self) -> f32 { self.base + self.modifier }
}
impl LuaUserData for Stat {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("base", |_, this| Ok(this.base));
        fields.add_field_method_set("base", |_, this, base| Ok(this.base = base));
        fields.add_field_method_get("mod", |_, this| Ok(this.modifier));
        fields.add_field_method_set("mod", |_, this, modifier| Ok(this.modifier = modifier));
        fields.add_field_method_get("total", |_, this| Ok(this.total()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Call, |_, this, ()| Ok(this.total()));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{{base = {}, mod = {}}}", this.base, this.modifier)));
    }
}
impl LuaMod for Stat {
    fn mod_name() -> &'static str { "Stat" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("new", lua.create_function(|_ctx, base| {
            Ok(Stat { base, modifier: 0. })
        })?)?;
        table.set("get", lua.create_function(|ctx, (entity, statname): (LuaEntity, String)| {
            if let Some(ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_entity(entity.inner()?) {
                if let Some(attributes) = ent.get::<Attributes>() {
                    return Ok(attributes.stats.get(&statname).cloned());
                }
            }
            Ok(None)
        })?)?;
        table.set("set", lua.create_function(|ctx, (entity, statname, stat): (LuaEntity, String, Stat)| {
            if let Some(mut ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity_mut(entity.inner()?) {
                if let Some(mut attributes) = ent.get_mut::<Attributes>() {
                    attributes.stats.insert(statname, stat);
                }
            }
            Ok(())
        })?)?;
        table.set("total", lua.create_function(|ctx, (entity, statname): (LuaEntity, String)| {
            if let Some(ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_entity(entity.inner()?) {
                if let Some(attributes) = ent.get::<Attributes>() {
                    return Ok(attributes.stats.get(&statname).map(Stat::total));
                }
            }
            Ok(None)
        })?)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Pool {
    pub base:     f32,
    pub current:  f32,
    #[serde(default)]
    pub modifier: f32,
}
impl Pool {
    pub fn cap(&self) -> f32 { self.base + self.modifier }
    pub fn ratio(&self) -> f32 { self.current / self.cap() }
}
impl Add<f32> for Pool {
    type Output = Pool;
    
    fn add(self, rhs: f32) -> Self::Output {
        Pool { current: (self.current + rhs).min(self.cap()).max(0.), ..self }
    }
}
impl Div<f32> for Pool {
    type Output = Pool;
    
    fn div(self, rhs: f32) -> Self::Output {
        Pool { current: (self.current / rhs).min(self.cap()).max(0.), ..self }
    }
}
impl Mul<f32> for Pool {
    type Output = Pool;
    
    fn mul(self, rhs: f32) -> Self::Output {
        Pool { current: (self.current * rhs).min(self.cap()).max(0.), ..self }
    }
}
impl Sub<f32> for Pool {
    type Output = Pool;
    
    fn sub(self, rhs: f32) -> Self::Output {
        Pool { current: (self.current - rhs).min(self.cap()).max(0.), ..self }
    }
}
impl LuaUserData for Pool {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("base", |_, this| Ok(this.base));
        fields.add_field_method_set("base", |_, this, base| Ok(this.base = base));
        fields.add_field_method_get("current", |_, this| Ok(this.current));
        fields.add_field_method_set("current", |_, this, current: f32| Ok(this.current = current.min(this.cap()).max(0.)));
        fields.add_field_method_get("mod", |_, this| Ok(this.modifier));
        fields.add_field_method_set("mod", |_, this, modifier| Ok(this.modifier = modifier));
        fields.add_field_method_get("ratio", |_, this| Ok(this.ratio()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, f: f32| Ok(*this + f));
        methods.add_meta_method(LuaMetaMethod::Call, |_, this, ()| Ok(this.current));
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, f: f32| Ok(*this / f));
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, f: f32| Ok(*this * f));
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, f: f32| Ok(*this - f));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("Pool {}/({} + {})", this.current, this.base, this.modifier)));
    }
}
impl LuaMod for Pool {
    fn mod_name() -> &'static str { "Pool" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("new", lua.create_function(|_ctx, base| {
            Ok(Pool { base, modifier: 0., current: base })
        })?)?;
        table.set("get", lua.create_function(|ctx, (entity, name): (LuaEntity, String)| {
            if let Some(ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_entity(entity.inner()?) {
                if let Some(attributes) = ent.get::<Attributes>() {
                    return Ok(attributes.stats.get(&name).cloned());
                }
            }
            Ok(None)
        })?)?;
        table.set("set", lua.create_function(|ctx, (entity, name, pool): (LuaEntity, String, Pool)| {
            if let Some(mut ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity_mut(entity.inner()?) {
                if let Some(mut attributes) = ent.get_mut::<Attributes>() {
                    attributes.pools.insert(name, pool);
                }
            }
            Ok(())
        })?)?;
        table.set("current", lua.create_function(|ctx, (entity, name): (LuaEntity, String)| {
            if let Some(ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_entity(entity.inner()?) {
                if let Some(attributes) = ent.get::<Attributes>() {
                    return Ok(attributes.pools.get(&name).map(|p| p.current));
                }
            }
            Ok(None)
        })?)?;
        table.set("cap", lua.create_function(|ctx, (entity, name): (LuaEntity, String)| {
            if let Some(ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_entity(entity.inner()?) {
                if let Some(attributes) = ent.get::<Attributes>() {
                    return Ok(attributes.pools.get(&name).map(Pool::cap));
                }
            }
            Ok(None)
        })?)?;
        table.set("ratio", lua.create_function(|ctx, (entity, name): (LuaEntity, String)| {
            if let Some(ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_entity(entity.inner()?) {
                if let Some(attributes) = ent.get::<Attributes>() {
                    return Ok(attributes.pools.get(&name).map(|p| p.ratio()));
                }
            }
            Ok(None)
        })?)?;
        Ok(())
    }
}