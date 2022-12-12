use std::{sync::Arc, collections::{HashMap}};

use bevy::{asset::*, prelude::*, reflect::{TypeUuid}};
use bevy_inspector_egui::InspectorOptions;
use mlua::prelude::*;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use serde::{Deserialize, Serialize};

use crate::scripting::{time::LuaTime, bevy_api::{LuaEntity, math::{LuaVec2, LuaVec3}}, color::RgbaColor, lua_to_string, LuaMod};

use super::palette::{DynColor};

pub struct InstanceRef {
    pub lock: RwLock<Lua>,
}
impl From<RwLock<Lua>> for InstanceRef {
    fn from(lock: RwLock<Lua>) -> Self {
        InstanceRef { lock }
    }
}
// This is unsafe; we shouldn't do this in the future
// The RwLock is a precaution, Lua's mutating behaviors can be done without &mut since it holds UnsafeCells
// So always take Write lock unless you're absolutely sure its safe and required for performance
unsafe impl Send for InstanceRef {}
unsafe impl Sync for InstanceRef {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect)]
pub enum InstanceKind {
    /// I want everyone using this script to have their own unique scope
    /// 
    /// Safest, but with the greatest impact on performance if used on a lot of entities
    /// 
    /// Scripts default to this
    Unique,
    /// I'm fine sharing the scope with everyone using this script
    /// 
    /// on_init is call in the same scope everytime a new entity with this script spawns
    /// 
    /// This has better performance when lots of entities are using the same script, but be careful with global variables
    /// and be aware that the `entity` global can change each call!
    /// 
    /// Enable for a script by starting the file with `--!shared`
    Shared,
    /// This scope has plenty of room for everyone, tovarish, regardless of script file (or lack thereof)
    /// 
    /// All the risks that shared has, plus more; I'm not sure why you'd want to use it,
    /// maybe if you're obsessive about premature optimization?
    /// 
    /// Enable for a script by starting the file with `--!collectivist`
    Collectivist,
}

#[derive(Clone, Debug, Eq, PartialEq, Reflect, TypeUuid)]
#[uuid = "100a1234-cb2e-46a7-8e36-4cb2fb671746"]
pub struct LuaScript {
    pub instance: InstanceKind,
    pub source:   String,
}

#[derive(Debug, Clone)]
pub struct LuaWorld {
    pub pointer: Arc<RwLock<*mut World>>,
}
unsafe impl Send for LuaWorld {}
unsafe impl Sync for LuaWorld {}
impl LuaWorld {
    pub unsafe fn new(world: &mut World) -> Self {
        LuaWorld { pointer: Arc::new(RwLock::new(world)) }
    }

    pub fn read(&self) -> MappedRwLockReadGuard<World> {
        RwLockReadGuard::map(self.pointer.try_read().expect(""), |w: &*mut World| unsafe { &**w })
    }

    pub fn write(&self) -> MappedRwLockWriteGuard<World> {
        RwLockWriteGuard::map(self.pointer.try_write().expect(""), |w: &mut *mut World| unsafe { &mut **w })
    }
}
impl LuaUserData for LuaWorld {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {
        
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {

    }
}

#[derive(Default)]
pub struct LuaScriptLoader;

impl AssetLoader for LuaScriptLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let source   = std::str::from_utf8(bytes)?.to_string();
            let instance = if source.starts_with("--!shared") {
                InstanceKind::Shared
            } else if source.starts_with("--!collectivist") {
                InstanceKind::Collectivist
            } else {
                InstanceKind::Unique
            };
            load_context.set_default_asset(LoadedAsset::new(LuaScript { instance, source }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["lua"]
    }
}

#[derive(Clone, Debug, FromReflect, Reflect)]
pub struct Hook {
    pub name: String,
    #[reflect(ignore)]
    pub args: ManyScriptVars,
}
impl Hook {
    // I have no idea why this doesn't compile but `exec` does
    // pub fn call<'lua, T>(&self, lua: &'lua RwLock<Lua>, entity: LuaEntity) -> Result<Option<T>, LuaError> where T: Clone + FromLua<'lua> {
    //     let lua = lua.write();
    //     lua.globals().set("entity", entity)?;
    //     if let Some(f) = lua.globals().get::<_, Option<LuaFunction>>(self.name.clone())? {
    //         Ok(Some(f.call(self.args.clone())?))
    //     } else { Ok(None) }
    // }

    pub fn exec<'lua>(&self, lua: &'lua RwLock<Lua>, entity: LuaEntity) -> Result<(), LuaError> {
        let lua = lua.write();
        lua.globals().set("entity", entity)?;
        if let Some(f) = lua.globals().get::<_, Option<LuaFunction>>(self.name.clone())? {
            f.call(self.args.clone())?;
        }
        Ok(())
    }

    pub fn log_err(&self, err: LuaError) {
        error!("{} threw {}", self.name, err);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Recipient {
    Entity(Entity),
    Script(String),
    Everyone,
    NoOne,
}

pub enum Any2<A, B> { A(A), B(B) }
impl<'lua, A, B> FromLua<'lua> for Any2<A, B> where A: FromLua<'lua>, B: FromLua<'lua> {
    fn from_lua(v: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        if let Some(a) = A::from_lua(v.clone(), lua).ok() {
            Ok(Any2::A(a))
        } else if let Some(b) = B::from_lua(v.clone(), lua).ok() {
            Ok(Any2::B(b))
        } else {
            Err(LuaError::RuntimeError(format!("Failed Any2 conversion")))
        }
    }
}
pub enum Any3<A, B, C> { A(A), B(B), C(C) }
impl<'lua, A, B, C> FromLua<'lua> for Any3<A, B, C> where A: FromLua<'lua>, B: FromLua<'lua>, C: FromLua<'lua> {
    fn from_lua(v: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        if let Some(a) = A::from_lua(v.clone(), lua).ok() {
            Ok(Any3::A(a))
        } else if let Some(b) = B::from_lua(v.clone(), lua).ok() {
            Ok(Any3::B(b))
        } else if let Some(c) = C::from_lua(v.clone(), lua).ok() {
            Ok(Any3::C(c))
        } else {
            Err(LuaError::RuntimeError(format!("Failed Any3 conversion")))
        }
    }
}

pub trait LuaReadable {
    fn with_read<F, R>(&self, f: F) -> R where F: FnOnce(&Lua) -> R;
}
impl LuaReadable for InstanceRef {
    fn with_read<F, R>(&self, f: F) -> R where F: FnOnce(&Lua) -> R {
        f(&self.lock.read())
    }
}
impl LuaReadable for Lua {
    fn with_read<F, R>(&self, f: F) -> R where F: FnOnce(&Lua) -> R {
        f(self)
    }
}

/* ********* */
// ScriptVar //
/* ********* */
// Ref-free lua values for serialization and transfer between instances

#[derive(Clone, Debug, Default, Deserialize, InspectorOptions, PartialEq, Serialize)]
pub enum ScriptVar {
    AnyUserTable(Vec<(ScriptVar, ScriptVar)>),
    Array(Vec<ScriptVar>),
    Bool(bool),
    Color(RgbaColor),
    DynColor(DynColor),
    Entity(u64),
    Int(i64),
    #[default]
    Nil,
    Num(f64),
    Str(String),
    Table(HashMap<String, ScriptVar>),
    Time(LuaTime),
    Vec2(Vec2),
    Vec3(Vec3),
}
impl<'lua> ToLua<'lua> for ScriptVar {
    fn to_lua(self, lua: &'lua Lua) -> Result<LuaValue<'lua>, LuaError> {
        match self {
            ScriptVar::AnyUserTable(pairs) => {
                let table = lua.create_table()?;
                for (k, v) in pairs {
                    table.set(k.to_lua(lua)?, v.to_lua(lua)?)?;
                }
                Ok(LuaValue::Table(table))
            },
            ScriptVar::Array(v)    => Ok(v.to_lua(lua)?),
            ScriptVar::Bool(b)     => Ok(LuaValue::Boolean(b)),
            ScriptVar::Color(c)    => Ok(c.to_lua(lua)?),
            ScriptVar::DynColor(c) => Ok(c.to_lua(lua)?),
            ScriptVar::Entity(u)   => Ok(LuaEntity::new(Entity::from_bits(u)).to_lua(lua)?),
            ScriptVar::Int(i)      => Ok(LuaValue::Integer(i)),
            ScriptVar::Nil         => Ok(LuaValue::Nil),
            ScriptVar::Num(i)      => Ok(LuaValue::Number(i)),
            ScriptVar::Str(s)      => Ok(s.to_lua(lua)?),
            ScriptVar::Table(t)    => Ok(t.to_lua(lua)?),
            ScriptVar::Time(t)     => Ok(t.to_lua(lua)?),
            ScriptVar::Vec2(v)     => Ok(LuaVec2::new(v).to_lua(lua)?),
            ScriptVar::Vec3(v)     => Ok(LuaVec3::new(v).to_lua(lua)?),
        }
    }
}
impl<'lua> FromLua<'lua> for ScriptVar {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> Result<Self, LuaError> {
        match lua_value {
            LuaValue::Boolean(b) => Ok(ScriptVar::Bool(b)),
            LuaValue::Integer(i) => Ok(ScriptVar::Int(i)),
            LuaValue::Number(n) => Ok(ScriptVar::Num(n)),
            LuaValue::Nil => Ok(ScriptVar::Nil),
            LuaValue::String(s) => Ok(ScriptVar::Str(s.to_str()?.to_string())),
            LuaValue::Table(t) => {
                let mut pairs = Vec::new();
                for p in t.pairs().into_iter() {
                    pairs.push(p?);
                }
                Ok(ScriptVar::AnyUserTable(pairs))
            },
            LuaValue::UserData(data) => {
                if data.is::<LuaEntity>() {
                    Ok(ScriptVar::Entity(data.borrow::<LuaEntity>()?.clone().0.to_bits()))
                } else if data.is::<LuaVec2>() {
                    Ok(ScriptVar::Vec2(data.borrow::<LuaVec2>()?.clone().0))
                } else if data.is::<LuaVec3>() {
                    Ok(ScriptVar::Vec3(data.borrow::<LuaVec3>()?.clone().0))
                } else if data.is::<RgbaColor>() {
                    Ok(ScriptVar::Color(data.borrow::<RgbaColor>()?.clone().into()))
                } else if data.is::<DynColor>() {
                    Ok(ScriptVar::DynColor(data.borrow::<DynColor>()?.clone().into()))
                } else {
                    let meta = data.get_metatable()?;
                    if let Some(function) = meta.get::<_, Option<LuaFunction>>(LuaMetaMethod::Custom("__script_var".into()))? {
                        function.call(LuaValue::UserData(data))
                    } else {
                        Err(LuaError::RuntimeError(format!("UserData {} had no __script_var metamethod", lua_to_string(LuaValue::UserData(data))?)))
                    }
                }
            },
            v => Err(LuaError::RuntimeError(format!("LuaValue {} has no conversion into ScriptVar", lua_to_string(v)?))),
        }
    }
}
impl LuaMod for ScriptVar {
    fn mod_name() -> &'static str { "Vars" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("all", lua.create_function(|ctx, entity: LuaEntity| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            if let Some(vars) = w.get::<LuaScriptVars>(entity.0) {
                Ok(Some(vars.0.clone().to_lua(ctx)?))
            } else { Ok(None) }
        })?)?;
        Ok(())
    }
}
impl Into<ScriptVar> for ()   { fn into(self) -> ScriptVar { ScriptVar::Nil } }
impl Into<ScriptVar> for bool { fn into(self) -> ScriptVar { ScriptVar::Bool(self) } }
impl Into<ScriptVar> for i8  { fn into(self) -> ScriptVar { ScriptVar::Int(self as i64) } }
impl Into<ScriptVar> for i16 { fn into(self) -> ScriptVar { ScriptVar::Int(self as i64) } }
impl Into<ScriptVar> for i32 { fn into(self) -> ScriptVar { ScriptVar::Int(self as i64) } }
impl Into<ScriptVar> for i64 { fn into(self) -> ScriptVar { ScriptVar::Int(self as i64) } }
impl Into<ScriptVar> for u8  { fn into(self) -> ScriptVar { ScriptVar::Int(self as i64) } }
impl Into<ScriptVar> for u16 { fn into(self) -> ScriptVar { ScriptVar::Int(self as i64) } }
impl Into<ScriptVar> for u32 { fn into(self) -> ScriptVar { ScriptVar::Int(self as i64) } }
impl Into<ScriptVar> for u64 { fn into(self) -> ScriptVar { ScriptVar::Int(self as i64) } }
impl Into<ScriptVar> for f32 { fn into(self) -> ScriptVar { ScriptVar::Num(self as f64) } }
impl Into<ScriptVar> for f64 { fn into(self) -> ScriptVar { ScriptVar::Num(self as f64) } }
impl Into<ScriptVar> for Entity { fn into(self) -> ScriptVar { ScriptVar::Entity(self.to_bits()) } }
impl Into<ScriptVar> for LuaTime { fn into(self) -> ScriptVar { ScriptVar::Time(self) } }
impl Into<ScriptVar> for String { fn into(self) -> ScriptVar { ScriptVar::Str(self) } }
impl Into<ScriptVar> for Vec2 { fn into(self) -> ScriptVar { ScriptVar::Vec2(self) } }
impl Into<ScriptVar> for Vec3 { fn into(self) -> ScriptVar { ScriptVar::Vec3(self) } }
impl<V> Into<ScriptVar> for HashMap<String, V> where V: Clone + Into<ScriptVar> {
    fn into(self) -> ScriptVar {
        let table = self.iter().map(|(k, v)| (k.clone(), <V as Into<ScriptVar>>::into(v.clone()))).collect();
        ScriptVar::Table(table)
    }
}
impl<V> Into<ScriptVar> for Option<V> where V: Clone + Into<ScriptVar> {
    fn into(self) -> ScriptVar {
        match self {
            Some(v) => v.into(),
            None    => ScriptVar::Nil,
        }
    }
}
impl<V> Into<ScriptVar> for Vec<V> where V: Clone + Into<ScriptVar> {
    fn into(self) -> ScriptVar {
        let vec = self.iter().map(|v| <V as Into<ScriptVar>>::into(v.clone())).collect();
        ScriptVar::Array(vec)
    }
}

#[derive(Clone, Debug, Default, Deserialize, InspectorOptions, PartialEq, Serialize)]
pub struct ManyScriptVars(pub Vec<ScriptVar>);
impl<'lua> ToLuaMulti<'lua> for ManyScriptVars {
    fn to_lua_multi(self, lua: &'lua Lua) -> Result<LuaMultiValue<'lua>, LuaError> {
        let mut mv = LuaMultiValue::new();
        for val in self.0.iter().rev() {
            mv.push_front(val.clone().to_lua(lua)?);
        }
        Ok(mv)
    }
}
pub const UNIT_PARAMS: ManyScriptVars = ManyScriptVars(vec![]);

#[derive(Clone, Component, Debug, Default, Deserialize, InspectorOptions, PartialEq, Serialize)]
pub struct LuaScriptVars(pub std::collections::HashMap<String, ScriptVar>);

impl LuaScriptVars {
    pub fn merge(&mut self, other: &Self) {
        self.0.extend(other.0.iter().map(|p| (p.0.clone(), p.1.clone())));
    }
}
impl<A> Into<ManyScriptVars> for (A,) where A: Into<ScriptVar> {
    fn into(self) -> ManyScriptVars {
        ManyScriptVars(vec![self.0.into()])
    }
}
impl<A, B> Into<ManyScriptVars> for (A, B) where A: Into<ScriptVar>, B: Into<ScriptVar> {
    fn into(self) -> ManyScriptVars {
        ManyScriptVars(vec![self.0.into(), self.1.into()])
    }
}
impl<A, B, C> Into<ManyScriptVars> for (A, B, C) where A: Into<ScriptVar>, B: Into<ScriptVar>, C: Into<ScriptVar> {
    fn into(self) -> ManyScriptVars {
        ManyScriptVars(vec![self.0.into(), self.1.into(), self.2.into()])
    }
}
impl<A, B, C, D> Into<ManyScriptVars> for (A, B, C, D) where A: Into<ScriptVar>, B: Into<ScriptVar>, C: Into<ScriptVar>, D: Into<ScriptVar> {
    fn into(self) -> ManyScriptVars {
        ManyScriptVars(vec![self.0.into(), self.1.into(), self.2.into(), self.3.into()])
    }
}
impl<A, B, C, D, E> Into<ManyScriptVars> for (A, B, C, D, E) where A: Into<ScriptVar>, B: Into<ScriptVar>, C: Into<ScriptVar>, D: Into<ScriptVar>, E: Into<ScriptVar> {
    fn into(self) -> ManyScriptVars {
        ManyScriptVars(vec![self.0.into(), self.1.into(), self.2.into(), self.3.into(), self.4.into()])
    }
}