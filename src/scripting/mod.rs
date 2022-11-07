use std::{sync::Mutex, collections::HashMap};
use ::std::time::Duration;

use bevy::{prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_scripting::{prelude::*, core::{event::ScriptLoaded}, lua::api::bevy::{LuaBevyAPIProvider, LuaEntity, LuaWorld},};
use iyes_loopless::prelude::FixedTimestepStage;
use serde::{Deserialize, Serialize};

use crate::data::{stat::{Stat, Pool}, material::TextureMaterial};
use crate::util::serialize::*;

use self::{event::{ON_UPDATE, ON_INIT}, color::RgbaColor};

pub mod color;
pub mod entity;
pub mod event;
pub mod geometry;
pub mod level;
pub mod log;
pub mod random;
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
            .add_stage_before(
                CoreStage::Update,
                "on_update",
                FixedTimestepStage::new(Duration::from_secs_f32(1. / 15.)).with_stage(on_update)
            )
            .add_script_handler_stage::<LuaScriptHost<()>, _, 0, 1>(CoreStage::PostUpdate)
            .add_script_host::<LuaScriptHost<()>, _>(CoreStage::PostUpdate)
            .add_api_provider::<LuaScriptHost<()>>(Box::new(LuaBevyAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(PreludeAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(color::ColorAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(entity::EntityAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(geometry::GeometryAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(level::LevelAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(log::LogAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(random::RandomAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(time::TimeAPIProvider))
            .add_system(send_on_init)
            ;
    }
}

pub trait LuaMod {
    fn mod_name() -> &'static str;

    fn register_defs(ctx: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error>;
}

pub fn init_luamod<T>(ctx: &Lua) -> Result<(), mlua::Error> where T: LuaMod {
    let mut table = ctx.create_table()?;
    T::register_defs(ctx, &mut table)?;
    ctx.globals().set(T::mod_name().to_string(), table)?;
    Ok(())
}

pub fn send_on_init(
    mut lua_events: PriorityEventWriter<LuaEvent<()>>,
    mut events: EventReader<ScriptLoaded>,
) {
    for script in events.iter() {
        lua_events.send(LuaEvent { hook_name: ON_INIT.into(), args: (), recipients: Recipients::ScriptID(script.sid) }, 1);
    }
}

pub fn send_on_update(mut events: PriorityEventWriter<LuaEvent<()>>) {
    events.send(
        LuaEvent {
            hook_name: ON_UPDATE.into(),
            args: (),
            recipients: Recipients::All,
        },
        1,
    )
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
        init_luamod::<ScriptVar>(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<TextureMaterial>(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<Pool>(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<Stat>(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

fn attach_prelude_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    ctx.globals().set("format", ctx.create_function(|_ctx, values: LuaMultiValue| {
        format_lua(values)
    })?)?;
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

#[derive(Clone, Debug, Default, Deserialize, Inspectable, PartialEq, Reflect, Serialize)]
pub enum ScriptVar {
    Bool(bool),
    Color(#[serde(deserialize_with = "deserialize_hex_color", serialize_with = "serialize_hex_color")] Color),
    Int(i64),
    #[default]
    Nil,
    Num(f64),
    String(String),
}
impl<'lua> ToLua<'lua> for ScriptVar {
    fn to_lua(self, lua: &'lua Lua) -> Result<LuaValue<'lua>, LuaError> {
        match self {
            ScriptVar::Bool(b) => Ok(LuaValue::Boolean(b)),
            ScriptVar::Color(c) => Ok(RgbaColor::from(c).to_lua(lua)?),
            ScriptVar::Int(i) => Ok(LuaValue::Integer(i)),
            ScriptVar::Nil         => Ok(LuaValue::Nil),
            ScriptVar::Num(i) => Ok(LuaValue::Number(i)),
            ScriptVar::String(s) => Ok(s.to_lua(lua)?),
        }
    }
}
impl LuaMod for ScriptVar {
    fn mod_name() -> &'static str { "Var" }
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

#[derive(Clone, Component, Debug, Default, Deserialize, Inspectable, PartialEq, Serialize)]
pub struct LuaScriptVars(pub HashMap<String, ScriptVar>);

impl LuaScriptVars {
    pub fn merge(&mut self, other: &Self) {
        self.0.extend(other.0.iter().map(|p| (p.0.clone(), p.1.clone())));
    }
}