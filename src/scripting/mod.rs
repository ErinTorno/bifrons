use std::{sync::Mutex, collections::{HashMap, HashSet}};
use ::std::time::Duration;

use bevy::{prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_scripting::{prelude::*, core::{event::ScriptLoaded}, lua::api::bevy::{LuaBevyAPIProvider, LuaEntity, LuaWorld, LuaVec3, LuaVec2, LuaQuat},};
use iyes_loopless::prelude::FixedTimestepStage;
use serde::{Deserialize, Serialize};

use crate::{data::{stat::{Stat, Pool}, material::{TextureMaterial, TexMatInfo}, input::ActionState, formlist::{FormList, InjectCommands}, level::Level, geometry::{Light, LightAnim, LightKind}}};
use crate::util::serialize::*;

use self::{event::{ON_UPDATE, ON_INIT}, color::RgbaColor, time::LuaTime, message::MessageQueue, registry::Registry};

pub mod color;
pub mod entity;
pub mod event;
pub mod level;
pub mod log;
pub mod message;
pub mod random;
pub mod registry;
pub mod time;

#[derive(Clone, Debug, Default)]
pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut on_update = SystemStage::single_threaded();
        on_update.add_system(send_on_update);

        app
            .register_inspectable::<ScriptVar>()
            .register_type::<ScriptVar>()
            .register_inspectable::<LuaScriptVars>()
            .init_resource::<LuaTime>()
            .init_resource::<MessageQueue>()
            .init_resource::<Registry>()
            .init_resource::<ScriptsInfo>()
            .add_stage_before(
                CoreStage::Update,
                "on_update",
                FixedTimestepStage::new(Duration::from_secs_f32(event::constants::ON_UPDATE_DELAY)).with_stage(on_update)
            )
            .add_script_handler_stage::<LuaScriptHost<ManyScriptVars>, _, 0, 1>(CoreStage::PostUpdate)
            .add_script_host::<LuaScriptHost<ManyScriptVars>, _>(CoreStage::PostUpdate)
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(LuaBevyAPIProvider))
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(PreludeAPIProvider))
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(color::ColorAPIProvider))
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(entity::EntityAPIProvider))
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(level::LevelAPIProvider))
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(log::LogAPIProvider))
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(message::MessageAPIProvider))
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(random::RandomAPIProvider))
            .add_api_provider::<LuaScriptHost<ManyScriptVars>>(Box::new(time::TimeAPIProvider))
            .add_system(send_on_init)
            .add_system(update_script_queue)
            ;
    }
}

pub fn register_lua_mods(lua: &Lua) -> Result<(), LuaError> {
    init_luamod::<ActionState>(lua)?;
    init_luamod::<FormList>(lua)?;
    init_luamod::<InjectCommands>(lua)?;
    init_luamod::<Light>(lua)?;
    init_luamod::<LightAnim>(lua)?;
    init_luamod::<LightKind>(lua)?;
    init_luamod::<Registry>(lua)?;
    init_luamod::<ScriptVar>(lua)?;
    init_luamod::<TextureMaterial>(lua)?;
    init_luamod::<Pool>(lua)?;
    init_luamod::<Stat>(lua)?;
    Ok(())
}

pub trait LuaMod {
    fn mod_name() -> &'static str;

    fn register_defs(ctx: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error>;
}

#[derive(Clone, Component)]
pub struct AwaitScript {
    pub script_ids: HashSet<u32>,
    pub event:      LuaEvent<ManyScriptVars>,
}

#[derive(Clone, Default)]
pub struct ScriptsInfo {
    pub loaded: HashSet<u32>,
}

pub fn init_luamod<T>(ctx: &Lua) -> Result<(), mlua::Error> where T: LuaMod {
    let mut table = ctx.create_table()?;
    T::register_defs(ctx, &mut table)?;
    ctx.globals().set(T::mod_name().to_string(), table)?;
    Ok(())
}

pub fn send_on_init(
    mut scripts_info: ResMut<ScriptsInfo>,
    mut lua_events: PriorityEventWriter<LuaEvent<ManyScriptVars>>,
    mut events: EventReader<ScriptLoaded>,
) {
    for script in events.iter() {
        scripts_info.loaded.insert(script.sid);
        lua_events.send(LuaEvent { hook_name: ON_INIT.into(), args: UNIT_PARAMS, recipients: Recipients::ScriptID(script.sid) }, 1);
    }
}

pub fn send_on_update(
    time:         Res<Time>,
    mut lua_time: ResMut<LuaTime>,
    mut events: PriorityEventWriter<LuaEvent<ManyScriptVars>>
) {
    let elapsed = time.seconds_since_startup();
    let delta = if lua_time.elapsed > 0. { elapsed - lua_time.elapsed } else { 0. };
    lua_time.elapsed = elapsed;
    lua_time.delta = delta;
    events.send(
        LuaEvent {
            hook_name: ON_UPDATE.into(),
            args: (lua_time.clone(),).into(),
            recipients: Recipients::All,
        },
        1,
    );
}

pub fn update_script_queue(
    mut commands: Commands,
    mut events:   PriorityEventWriter<LuaEvent<ManyScriptVars>>,
    scripts_info: Res<ScriptsInfo>,
    mut messages: ResMut<MessageQueue>,
    query:        Query<(Entity, &AwaitScript)>,
) {
    for (entity, awaiting) in query.iter() {        
        if awaiting.script_ids.is_subset(&scripts_info.loaded) {
            commands.entity(entity)
                .remove::<AwaitScript>();
            events.send(awaiting.event.clone(), 1);
        }
    }

    while let Some(event) = messages.events.pop() {
        events.send(event, 1);
    }
}

// Default API

#[derive(Default)]
pub struct PreludeAPIProvider;

impl APIProvider for PreludeAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_prelude_lua(ctx).map_err(ScriptError::new_other)?;
        register_lua_mods(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

fn attach_prelude_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    ctx.globals().set("format", ctx.create_function(|_ctx, values: LuaMultiValue| {
        format_lua(values)
    })?)?;
    ctx.globals().set("PI", std::f64::consts::PI)?;
    ctx.globals().set("string", ctx.create_function(|_ctx, value: LuaValue| {
        lua_to_string(value)
    })?)?;
    ctx.globals().set("finite_or", ctx.create_function(|_ctx, (n, def): (f32, f32)| {
        Ok(if n.is_finite() { n } else { def })
    })?)?;
    Ok(())
}

pub fn lua_to_string(value: LuaValue) -> Result<String, mlua::Error> {
    match value {
        Value::Boolean(b) => Ok(b.to_string()),
        Value::Error(e)  => Ok(e.to_string()),
        Value::Function(f)  => {
            let info = f.info();
            let s = |opt, name: &str| if let Some(vec) = opt {
                String::from_utf8(vec).map_err(|e| mlua::Error::DeserializeError(e.to_string()))
            } else { Ok(format!("no_{}", name)) };
            Ok(format!("fn# {}@{}:{}-{}", s(info.name, "name")?, s(info.short_src, "src")?, info.line_defined, info.last_line_defined))
        },
        Value::Integer(i)  => Ok(i.to_string()),
        Value::LightUserData(data) => Ok(format!("light_user_data#{:?}", data)),
        Value::Nil              => Ok("nil".to_string()),
        Value::Number(n)   => Ok(n.to_string()),
        Value::String(s) => Ok(s.to_str()?.to_string()),
        Value::Table(table) => {
            let mut str = String::new();
            str += "{";
            let mut first = true;
            for pair in table.pairs() {
                let (k, v) = pair?;
                let key = lua_to_string(k)?;
                let val = lua_to_string(v)?;
                str += format!("{}{} = {}", if first { "" } else { ", " }, key, val).as_str();
                first = false;
            }
            str += "}";
            Ok(str)
        },
        Value::Thread(_) => Ok("#thread".to_string()),
        Value::UserData(data) => {
            let meta = data.get_metatable()?;
            if meta.contains(LuaMetaMethod::ToString)? {
                let tostring: LuaFunction = meta.get(LuaMetaMethod::ToString)?;
                Ok(tostring.call(Value::UserData(data))?)
            } else { Ok("#userdata".to_string()) }
        },
        // Value::Vector(x, y, z) => Ok(format!("{{x: {}, y: {}, z: {}}}", x, y, z)),
    }
}

pub fn format_lua(values: LuaMultiValue) -> Result<String, LuaError> {
    let mut s = String::new();
    let mut format_str = None;
    let mut params = Vec::new();
    for val in values.iter() {
        if format_str.is_none() {
            format_str = Some(lua_to_string(val.clone())?);
        } else {
            params.push(lua_to_string(val.clone())?);
        }
    }
    if let Some(format_str) = format_str {
        let mut idx = 0;
        for part in format_str.split("{}") {
            s += part;
            if idx < params.len() {
                s += params[idx].as_str();
                idx += 1;
            }
        }
    }
    Ok(s)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetKind {
    FormList,
    Level,
    Material,
}

#[derive(Clone, Debug)]
pub struct LuaHandle {
    pub handle: HandleUntyped,
    pub kind:   AssetKind,
}
impl LuaHandle {
    pub fn get_path(&self, asset_server: &AssetServer) -> Option<String> {
        match self.kind {
            AssetKind::FormList => asset_server.get_handle_path(self.handle.typed_weak::<FormList>()),
            AssetKind::Level    => asset_server.get_handle_path(self.handle.typed_weak::<Level>()),
            AssetKind::Material => asset_server.get_handle_path(self.handle.typed_weak::<StandardMaterial>()),
        }.map(|p| p.path().to_string_lossy().to_string())
    }
}
impl From<Handle<FormList>> for LuaHandle {
    fn from(handle: Handle<FormList>) -> Self {
        LuaHandle { handle: handle.clone_weak_untyped(), kind: AssetKind::FormList }
    }
}
impl From<Handle<Level>> for LuaHandle {
    fn from(handle: Handle<Level>) -> Self {
        LuaHandle { handle: handle.clone_weak_untyped(), kind: AssetKind::Level }
    }
}
impl From<Handle<StandardMaterial>> for LuaHandle {
    fn from(handle: Handle<StandardMaterial>) -> Self {
        LuaHandle { handle: handle.clone_weak_untyped(), kind: AssetKind::Material }
    }
}
impl LuaUserData for LuaHandle {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, this| Ok(match this.kind {
            AssetKind::FormList => "formlist",
            AssetKind::Level => "level",
            AssetKind::Material => "material",
        }));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("#handle<{:?}>{{id = {:?}}}", this.kind, this.handle.id)));
        methods.add_method("get", |lua: &Lua, this: &LuaHandle, ()| {
            match this.kind {
                AssetKind::FormList => {
                    let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                    let w = world.read();
                    let assets = w.get_resource::<Assets<FormList>>().unwrap();
                    if let Some(asset) = assets.get(&this.handle.clone().typed()) {
                        Ok(Some(asset.clone().to_lua(lua)?))
                    } else { Ok(None) }
                },
                AssetKind::Level => Err(LuaError::RuntimeError("Cannot load Level assets into Lua; see the Level module".to_string())),
                AssetKind::Material => {
                    let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                    let w = world.read();
                    if let Some(tex_mat_info) = w.get_resource::<TexMatInfo>() {
                        if let Some(texmat) = tex_mat_info.materials.get(&this.handle.clone().typed()) {
                            Ok(Some(texmat.clone().to_lua(lua)?))
                        } else { Ok(None) }
                    } else {
                        Err(LuaError::RuntimeError(format!("TexMatInfo not found")))
                    }
                },
            }
        });
        methods.add_method("is_loaded", |lua, this, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            match this.kind {
                AssetKind::FormList => {
                    let assets = w.get_resource::<Assets<FormList>>().unwrap();
                    Ok(assets.contains(&this.handle.clone().typed()))
                },
                AssetKind::Level => {
                    let assets = w.get_resource::<Assets<Level>>().unwrap();
                    Ok(assets.contains(&this.handle.clone().typed()))
                },
                AssetKind::Material => {
                    let assets = w.get_resource::<Assets<StandardMaterial>>().unwrap();
                    Ok(assets.contains(&this.handle.clone().typed()))
                },
            }
        });
        methods.add_method("path", |lua, this, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let asset_server = w.get_resource::<AssetServer>().unwrap();
            Ok(this.get_path(asset_server))
        });
    }
}

#[derive(Clone, Debug, Default, Deserialize, Inspectable, PartialEq, Reflect, Serialize)]
pub enum ScriptVar {
    AnyUserTable(Vec<(ScriptVar, ScriptVar)>),
    Array(Vec<ScriptVar>),
    Bool(bool),
    Color(#[serde(deserialize_with = "deserialize_hex_color", serialize_with = "serialize_hex_color")] Color),
    Entity(u64),
    Int(i64),
    #[default]
    Nil,
    Num(f64),
    Quat(Quat),
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
            ScriptVar::Array(v) => Ok(v.to_lua(lua)?),
            ScriptVar::Bool(b) => Ok(LuaValue::Boolean(b)),
            ScriptVar::Color(c) => Ok(RgbaColor::from(c).to_lua(lua)?),
            ScriptVar::Entity(u) => Ok(LuaEntity::new(Entity::from_bits(u)).to_lua(lua)?),
            ScriptVar::Int(i) => Ok(LuaValue::Integer(i)),
            ScriptVar::Nil         => Ok(LuaValue::Nil),
            ScriptVar::Num(i) => Ok(LuaValue::Number(i)),
            ScriptVar::Quat(q) => Ok(LuaQuat::new(q).to_lua(lua)?),
            ScriptVar::Str(s) => Ok(s.to_lua(lua)?),
            ScriptVar::Table(t) => Ok(t.to_lua(lua)?),
            ScriptVar::Time(t) => Ok(t.to_lua(lua)?),
            ScriptVar::Vec2(v) => Ok(LuaVec2::new(v).to_lua(lua)?),
            ScriptVar::Vec3(v) => Ok(LuaVec3::new(v).to_lua(lua)?),
        }
    }
}
impl<'lua> FromLua<'lua> for ScriptVar {
    fn from_lua(lua_value: Value<'lua>, lua: &'lua Lua) -> Result<Self, LuaError> {
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
                    Ok(ScriptVar::Entity(data.borrow::<LuaEntity>()?.clone().inner()?.to_bits()))
                }else if data.is::<LuaVec2>() {
                    Ok(ScriptVar::Vec2(data.borrow::<LuaVec2>()?.clone().inner()?))
                } else if data.is::<LuaVec3>() {
                    Ok(ScriptVar::Vec3(data.borrow::<LuaVec3>()?.clone().inner()?))
                } else if data.is::<LuaQuat>() {
                    Ok(ScriptVar::Quat(data.borrow::<LuaQuat>()?.clone().inner()?))
                } else if data.is::<RgbaColor>() {
                    Ok(ScriptVar::Color(data.borrow::<RgbaColor>()?.clone().into()))
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
            if let Some(vars) = w.get::<LuaScriptVars>(entity.inner()?) {
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
impl Into<ScriptVar> for Quat { fn into(self) -> ScriptVar { ScriptVar::Quat(self) } }
impl Into<ScriptVar> for Vec2 { fn into(self) -> ScriptVar { ScriptVar::Vec2(self) } }
impl Into<ScriptVar> for Vec3 { fn into(self) -> ScriptVar { ScriptVar::Vec3(self) } }
impl<V> Into<ScriptVar> for HashMap<String, V> where V: Clone + Into<ScriptVar> {
    fn into(self) -> ScriptVar {
        let table = self.iter().map(|(k, v)| (k.clone(), <V as Into<ScriptVar>>::into(v.clone()))).collect();
        ScriptVar::Table(table)
    }
}
impl<V> Into<ScriptVar> for Vec<V> where V: Clone + Into<ScriptVar> {
    fn into(self) -> ScriptVar {
        let vec = self.iter().map(|v| <V as Into<ScriptVar>>::into(v.clone())).collect();
        ScriptVar::Array(vec)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Inspectable, PartialEq, Serialize)]
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

#[derive(Clone, Component, Debug, Default, Deserialize, Inspectable, PartialEq, Serialize)]
pub struct LuaScriptVars(pub HashMap<String, ScriptVar>);

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