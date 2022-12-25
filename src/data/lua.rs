use std::{sync::Arc, collections::{HashMap}};

use bevy::{asset::*, prelude::*, reflect::{TypeUuid}};
use bevy_inspector_egui::Inspectable;
use mlua::prelude::*;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use serde::{Deserialize, Serialize};

use crate::scripting::{time::LuaTime, bevy_api::{LuaEntity, math::{LuaVec2, LuaVec3}, handle::LuaHandle}, color::RgbaColor, lua_to_string, LuaMod};

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
        RwLockReadGuard::map(self.pointer.try_read().expect("Unable to acquire world read lock"), |w: &*mut World| unsafe { &**w })
    }

    pub fn write(&self) -> MappedRwLockWriteGuard<World> {
        RwLockWriteGuard::map(self.pointer.try_write().expect("Unable to acquire world write lock"), |w: &mut *mut World| unsafe { &mut **w })
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

#[derive(Clone, Debug)]
pub struct Hook {
    pub name: String,
    pub args: ManyTransVars,
}
impl Hook {
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

#[derive(Clone, Debug, Default, Deserialize, Inspectable, PartialEq, Serialize)]
pub enum ScriptVar {
    AnyUserTable(Vec<(ScriptVar, ScriptVar)>),
    Array(Vec<ScriptVar>),
    Bool(bool),
    Color(DynColor),
    Int(i64),
    #[default]
    Nil,
    Num(f64),
    Rgba(RgbaColor),
    Str(String),
    Table(HashMap<String, ScriptVar>),
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
            ScriptVar::Int(i)      => Ok(LuaValue::Integer(i)),
            ScriptVar::Nil         => Ok(LuaValue::Nil),
            ScriptVar::Num(i)      => Ok(LuaValue::Number(i)),
            ScriptVar::Rgba(c)     => Ok(c.to_lua(lua)?),
            ScriptVar::Str(s)      => Ok(s.to_lua(lua)?),
            ScriptVar::Table(t)    => Ok(t.to_lua(lua)?),
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
                if data.is::<LuaVec2>() {
                    Ok(ScriptVar::Vec2(data.borrow::<LuaVec2>()?.clone().0))
                } else if data.is::<LuaVec3>() {
                    Ok(ScriptVar::Vec3(data.borrow::<LuaVec3>()?.clone().0))
                } else if data.is::<RgbaColor>() {
                    Ok(ScriptVar::Rgba(data.borrow::<RgbaColor>()?.clone().into()))
                } else if data.is::<DynColor>() {
                    Ok(ScriptVar::Color(data.borrow::<DynColor>()?.clone().into()))
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
impl From<()> for ScriptVar      { fn from(_: ())          -> Self { ScriptVar::Nil } }
impl From<bool> for ScriptVar    { fn from(value: bool)    -> Self { ScriptVar::Bool(value) } }
impl From<i8> for ScriptVar      { fn from(value: i8)      -> Self { ScriptVar::Int(value as i64) } }
impl From<i16> for ScriptVar     { fn from(value: i16)     -> Self { ScriptVar::Int(value as i64) } }
impl From<i32> for ScriptVar     { fn from(value: i32)     -> Self { ScriptVar::Int(value as i64) } }
impl From<i64> for ScriptVar     { fn from(value: i64)     -> Self { ScriptVar::Int(value as i64) } }
impl From<u8> for ScriptVar      { fn from(value: u8)      -> Self { ScriptVar::Int(value as i64) } }
impl From<u16> for ScriptVar     { fn from(value: u16)     -> Self { ScriptVar::Int(value as i64) } }
impl From<u32> for ScriptVar     { fn from(value: u32)     -> Self { ScriptVar::Int(value as i64) } }
impl From<u64> for ScriptVar     { fn from(value: u64)     -> Self { ScriptVar::Int(value as i64) } }
impl From<f32> for ScriptVar     { fn from(value: f32)     -> Self { ScriptVar::Num(value as f64) } }
impl From<f64> for ScriptVar     { fn from(value: f64)     -> Self { ScriptVar::Num(value as f64) } }
impl From<String> for ScriptVar  { fn from(value: String)  -> Self { ScriptVar::Str(value) } }
impl From<Vec2> for ScriptVar    { fn from(value: Vec2)    -> Self { ScriptVar::Vec2(value) } }
impl From<Vec3> for ScriptVar    { fn from(value: Vec3)    -> Self { ScriptVar::Vec3(value) } }
impl<V> From<HashMap<String, V>> for ScriptVar where V: Clone + Into<ScriptVar> {
    fn from(value: HashMap<String, V>) -> Self {
        let table = value.into_iter().map(|(k, v)| (k, v.into())).collect();
        ScriptVar::Table(table)
    }
}
impl<V> From<Option<V>> for ScriptVar where V: Clone + Into<ScriptVar> {
    fn from(value: Option<V>) -> Self {
        match value {
            Some(v) => v.into(),
            None    => ScriptVar::Nil,
        }
    }
}
impl<V> From<Vec<V>> for ScriptVar where V: Clone + Into<ScriptVar> {
    fn from(value: Vec<V>) -> Self {
        let vec = value.into_iter().map(|v| v.into()).collect();
        ScriptVar::Array(vec)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ManyTransVars(pub Vec<TransVar>);
impl<'lua> ToLuaMulti<'lua> for ManyTransVars {
    fn to_lua_multi(self, lua: &'lua Lua) -> Result<LuaMultiValue<'lua>, LuaError> {
        let mut mv = LuaMultiValue::new();
        for val in self.0.iter().rev() {
            mv.push_front(val.clone().to_lua(lua)?);
        }
        Ok(mv)
    }
}
impl<A> Into<ManyTransVars> for (A,) where A: Into<TransVar> {
    fn into(self) -> ManyTransVars {
        ManyTransVars(vec![self.0.into()])
    }
}
impl<A, B> Into<ManyTransVars> for (A, B) where A: Into<TransVar>, B: Into<TransVar> {
    fn into(self) -> ManyTransVars {
        ManyTransVars(vec![self.0.into(), self.1.into()])
    }
}
impl<A, B, C> Into<ManyTransVars> for (A, B, C) where A: Into<TransVar>, B: Into<TransVar>, C: Into<TransVar> {
    fn into(self) -> ManyTransVars {
        ManyTransVars(vec![self.0.into(), self.1.into(), self.2.into()])
    }
}
impl<A, B, C, D> Into<ManyTransVars> for (A, B, C, D) where A: Into<TransVar>, B: Into<TransVar>, C: Into<TransVar>, D: Into<TransVar> {
    fn into(self) -> ManyTransVars {
        ManyTransVars(vec![self.0.into(), self.1.into(), self.2.into(), self.3.into()])
    }
}
impl<A, B, C, D, E> Into<ManyTransVars> for (A, B, C, D, E) where A: Into<TransVar>, B: Into<TransVar>, C: Into<TransVar>, D: Into<TransVar>, E: Into<TransVar> {
    fn into(self) -> ManyTransVars {
        ManyTransVars(vec![self.0.into(), self.1.into(), self.2.into(), self.3.into(), self.4.into()])
    }
}

#[derive(Clone, Component, Debug, Default, Deserialize, Inspectable, PartialEq, Serialize)]
pub struct LuaScriptVars(pub std::collections::HashMap<String, ScriptVar>);

impl LuaScriptVars {
    pub fn merge(&mut self, other: &Self) {
        self.0.extend(other.0.iter().map(|p| (p.0.clone(), p.1.clone())));
    }
}

// Transfer

// For type we'll also want to transfer, but aren't serializable
// Or are types that aren't consistent between runs (like Entities and Handles)
#[derive(Clone, Debug, PartialEq)]
pub enum TransVar {
    AnyUserTable(Vec<(TransVar, TransVar)>),
    Entity(u64),
    Handle(LuaHandle),
    Time(LuaTime),
    Var(ScriptVar),
}
impl Default for TransVar {
    fn default() -> Self {
        TransVar::Var(ScriptVar::Nil)
    }
}
impl TransVar {
    pub fn try_handle_image(&self) -> Option<Handle<Image>> {
        match self {
            TransVar::Handle(handle) => handle.get_image(),
            _ => None,
        }
    }
}
impl<'lua> FromLua<'lua> for TransVar {
    fn from_lua(lua_value: LuaValue<'lua>, _lua: &'lua Lua) -> Result<Self, LuaError> {
        match lua_value {
            LuaValue::Table(t) => {
                let mut pairs = Vec::new();
                for p in t.pairs().into_iter() {
                    pairs.push(p?);
                }
                Ok(TransVar::AnyUserTable(pairs))
            },
            LuaValue::UserData(data) => {
                if data.is::<LuaEntity>() {
                    Ok(TransVar::Entity(data.borrow::<LuaEntity>()?.clone().0.to_bits()))
                } else if data.is::<LuaHandle>() {
                    Ok(TransVar::Handle(data.borrow::<LuaHandle>()?.clone()))
                } else if data.is::<LuaTime>() {
                    Ok(TransVar::Time(data.borrow::<LuaTime>()?.clone()))
                } else {
                    Ok(TransVar::Var(ScriptVar::from_lua(LuaValue::UserData(data), _lua)?))
                }
            },
            v => Ok(TransVar::Var(ScriptVar::from_lua(v, _lua)?)),
        }
    }
}
impl<'lua> ToLua<'lua> for TransVar {
    fn to_lua(self, lua: &'lua Lua) -> Result<LuaValue<'lua>, LuaError> {
        match self {
            TransVar::AnyUserTable(pairs) => {
                let table = lua.create_table()?;
                for (k, v) in pairs {
                    table.set(k.to_lua(lua)?, v.to_lua(lua)?)?;
                }
                Ok(LuaValue::Table(table))
            },
            TransVar::Entity(u) => Ok(LuaEntity::new(Entity::from_bits(u)).to_lua(lua)?),
            TransVar::Handle(h) => Ok(h.to_lua(lua)?),
            TransVar::Time(t)   => Ok(t.to_lua(lua)?),
            TransVar::Var(v)    => Ok(v.to_lua(lua)?),
        }
    }
}
impl From<()> for TransVar      { fn from(_: ())          -> Self { TransVar::Var(ScriptVar::Nil) } }
impl From<bool> for TransVar    { fn from(value: bool)    -> Self { TransVar::Var(ScriptVar::Bool(value)) } }
impl From<i8> for TransVar      { fn from(value: i8)      -> Self { TransVar::Var(ScriptVar::Int(value as i64)) } }
impl From<i16> for TransVar     { fn from(value: i16)     -> Self { TransVar::Var(ScriptVar::Int(value as i64)) } }
impl From<i32> for TransVar     { fn from(value: i32)     -> Self { TransVar::Var(ScriptVar::Int(value as i64)) } }
impl From<i64> for TransVar     { fn from(value: i64)     -> Self { TransVar::Var(ScriptVar::Int(value as i64)) } }
impl From<u8> for TransVar      { fn from(value: u8)      -> Self { TransVar::Var(ScriptVar::Int(value as i64)) } }
impl From<u16> for TransVar     { fn from(value: u16)     -> Self { TransVar::Var(ScriptVar::Int(value as i64)) } }
impl From<u32> for TransVar     { fn from(value: u32)     -> Self { TransVar::Var(ScriptVar::Int(value as i64)) } }
impl From<u64> for TransVar     { fn from(value: u64)     -> Self { TransVar::Var(ScriptVar::Int(value as i64)) } }
impl From<f32> for TransVar     { fn from(value: f32)     -> Self { TransVar::Var(ScriptVar::Num(value as f64)) } }
impl From<f64> for TransVar     { fn from(value: f64)     -> Self { TransVar::Var(ScriptVar::Num(value as f64)) } }
impl From<Entity> for TransVar  { fn from(value: Entity)  -> Self { TransVar::Entity(value.to_bits()) } }
impl From<LuaTime> for TransVar { fn from(value: LuaTime) -> Self { TransVar::Time(value) } }
impl From<String> for TransVar  { fn from(value: String)  -> Self { TransVar::Var(ScriptVar::Str(value)) } }
impl From<Vec2> for TransVar    { fn from(value: Vec2)    -> Self { TransVar::Var(ScriptVar::Vec2(value)) } }
impl From<Vec3> for TransVar    { fn from(value: Vec3)    -> Self { TransVar::Var(ScriptVar::Vec3(value)) } }
impl<V> From<Option<V>> for TransVar where V: Clone + Into<TransVar> {
    fn from(value: Option<V>) -> Self {
        match value {
            Some(v) => v.into(),
            None    => TransVar::default(),
        }
    }
}
impl<K, V> From<HashMap<K, V>> for TransVar where K: Into<TransVar>, V: Into<TransVar> {
    fn from(value: HashMap<K, V>) -> Self {
        let mut vec = Vec::new();
        for (k, v) in value {
            vec.push((k.into(), v.into()));
        }
        TransVar::AnyUserTable(vec)
    }
}

// *********************** //
// TryFrom out of TransVar //
// *********************** //

// Primitives
impl TryFrom<TransVar> for bool {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> {
        match value {
            TransVar::Var(ScriptVar::Bool(b)) => Ok(b),
            TransVar::Var(ScriptVar::Nil)     => Ok(false),
            _ => Ok(true),
        }
    }
}
impl TryFrom<TransVar> for i64 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> {
        match value {
            TransVar::Var(ScriptVar::Int(i)) => Ok(i),
            TransVar::Var(ScriptVar::Num(n))     => Ok(n as i64),
            _ => Err(format!("Not a Number {:?}", value)),
        }
    }
}
impl TryFrom<TransVar> for i8 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(i64::try_from(value)? as i8) }
}
impl TryFrom<TransVar> for i16 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(i64::try_from(value)? as i16) }
}
impl TryFrom<TransVar> for i32 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(i64::try_from(value)? as i32) }
}
impl TryFrom<TransVar> for u8 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(i64::try_from(value)? as u8) }
}
impl TryFrom<TransVar> for u16 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(i64::try_from(value)? as u16) }
}
impl TryFrom<TransVar> for u32 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(i64::try_from(value)? as u32) }
}
impl TryFrom<TransVar> for u64 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(i64::try_from(value)? as u64) }
}
impl TryFrom<TransVar> for usize {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(i64::try_from(value)? as usize) }
}
impl TryFrom<TransVar> for f64 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> {
        match value {
            TransVar::Var(ScriptVar::Int(i)) => Ok(i as f64),
            TransVar::Var(ScriptVar::Num(n)) => Ok(n),
            _ => Err(format!("Not a Number {:?}", value)),
        }
    }
}
impl TryFrom<TransVar> for f32 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> { Ok(f64::try_from(value)? as f32) }
}
impl TryFrom<TransVar> for Vec2 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> {
        match value {
            TransVar::Var(ScriptVar::Vec2(n)) => Ok(n),
            _ => Err(format!("Not a Vec2 {:?}", value)),
        }
    }
}
impl TryFrom<TransVar> for Vec3 {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> {
        match value {
            TransVar::Var(ScriptVar::Vec3(n)) => Ok(n),
            _ => Err(format!("Not a Vec3 {:?}", value)),
        }
    }
}

impl TryFrom<TransVar> for Entity {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> {
        match value {
            TransVar::Entity(b) => Ok(Entity::from_bits(b)),
            _ => Err(format!("Not an Entity {:?}", value)),
        }
    }
}

// Colors
impl TryFrom<TransVar> for DynColor {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> {
        match value {
            TransVar::Var(ScriptVar::Color(c)) => Ok(c.clone()),
            TransVar::Var(ScriptVar::Rgba(rgba)) => Ok(DynColor::Custom(rgba)),
            _ => Err(format!("Not a Color {:?}", value)),
        }
    }
}

// Handles
impl TryFrom<TransVar> for Handle<Image> {
    type Error = String;
    fn try_from(value: TransVar) -> Result<Self, Self::Error> {
        match value {
            TransVar::Handle(h) => h.get_image().ok_or_else(|| format!("Handle not of type Image: {:?}", h)),
            _ => Err(format!("Not a Handle<Image> {:?}", value)),
        }
    }
}