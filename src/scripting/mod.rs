use bevy::{prelude::*, asset::FileAssetIo};
use mlua::prelude::*;

use crate::{data::{stat::{Stat, Pool}, material::{TextureMaterial,}, input::ActionState, formlist::{FormList, InjectCommands}, geometry::{Light, LightAnim, LightKind}, lua::{ScriptVar}, palette::{Palette, DynColor}}};

use self::{color::RgbaColor, time::LuaTime, entity::{LuaQuery, EntityAPI}, level::LevelAPI, random::RandomAPI, log::LogAPI, bevy_api::{math::{LuaVec2, LuaVec3, MathAPI}, image::ImageAPI}, ui::{elem::{UIAPI}, atom::{LuaAtom}, text::{TextBuilder, TextStyle}, font::UIFont}, file::FileAPI};

pub mod bevy_api;
pub mod color;
pub mod entity;
pub mod event;
pub mod file;
pub mod level;
pub mod log;
pub mod message;
pub mod random;
pub mod time;
pub mod ui;

pub fn register_lua_mods(lua: &Lua) -> Result<(), LuaError> {
    init_luamod::<ActionState>(lua)?;
    init_luamod::<DynColor>(lua)?;
    init_luamod::<UIFont>(lua)?;
    init_luamod::<FormList>(lua)?;
    init_luamod::<EntityAPI>(lua)?;
    init_luamod::<FileAPI>(lua)?;
    init_luamod::<ImageAPI>(lua)?;
    init_luamod::<InjectCommands>(lua)?;
    init_luamod::<LuaAtom>(lua)?;
    init_luamod::<LuaQuery>(lua)?;
    init_luamod::<LuaTime>(lua)?;
    init_luamod::<LuaVec2>(lua)?;
    init_luamod::<LuaVec3>(lua)?;
    init_luamod::<LevelAPI>(lua)?;
    init_luamod::<Light>(lua)?;
    init_luamod::<LightAnim>(lua)?;
    init_luamod::<LightKind>(lua)?;
    init_luamod::<LogAPI>(lua)?;
    init_luamod::<MathAPI>(lua)?;
    init_luamod::<Palette>(lua)?;
    init_luamod::<RandomAPI>(lua)?;
    init_luamod::<ScriptVar>(lua)?;
    init_luamod::<TextBuilder>(lua)?;
    init_luamod::<TextStyle>(lua)?;
    init_luamod::<TextureMaterial>(lua)?;
    init_luamod::<Pool>(lua)?;
    init_luamod::<RgbaColor>(lua)?;
    init_luamod::<Stat>(lua)?;
    init_luamod::<UIAPI>(lua)?;
    attach_prelude_lua(lua)?;
    Ok(())
}

pub trait LuaMod {
    fn mod_name() -> &'static str;

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error>;
}

pub fn init_luamod<T>(lua: &Lua) -> Result<(), mlua::Error> where T: LuaMod {
    let mut table = lua.create_table()?;
    T::register_defs(lua, &mut table)?;
    lua.globals().set(T::mod_name().to_string(), table)?;
    Ok(())
}

// Default API

fn attach_prelude_lua(lua: &Lua) -> Result<(), mlua::Error> {
    let mut buf = FileAssetIo::get_base_path();
    buf.push("assets");
    let package: LuaTable = lua.globals().get("package")?;
    package.set("path", format!("{}/?.lua", buf.as_os_str().to_string_lossy()))?;

    lua.globals().set("format", lua.create_function(|_lua, values: LuaMultiValue| {
        format_lua(values)
    })?)?;
    lua.globals().set("string", lua.create_function(|_lua, value: LuaValue| {
        lua_to_string(value)
    })?)?;
    lua.set_warning_function(|_, str, _| {
        error!("{:?}", str);
        Ok(())
    });
    Ok(())
}

pub fn lua_to_string(value: LuaValue) -> Result<String, LuaError> {
    match value {
        LuaValue::Boolean(b) => Ok(b.to_string()),
        LuaValue::Error(e)  => Ok(e.to_string()),
        LuaValue::Function(f)  => {
            let info = f.info();
            let s = |opt, name: &str| if let Some(vec) = opt {
                String::from_utf8(vec).map_err(|e| LuaError::DeserializeError(e.to_string()))
            } else { Ok(format!("no_{}", name)) };
            Ok(format!("fn# {}@{}:{}-{}", s(info.name, "name")?, s(info.short_src, "src")?, info.line_defined, info.last_line_defined))
        },
        LuaValue::Integer(i)  => Ok(i.to_string()),
        LuaValue::LightUserData(data) => Ok(format!("light_user_data#{:?}", data)),
        LuaValue::Nil              => Ok("nil".to_string()),
        LuaValue::Number(n)   => Ok(n.to_string()),
        LuaValue::String(s) => Ok(s.to_str()?.to_string()),
        LuaValue::Table(table) => {
            let meta = table.get_metatable();
            if meta.is_some() && meta.as_ref().unwrap().contains_key(LuaMetaMethod::ToString.name())? {
                let tostring: LuaFunction = meta.unwrap().get(LuaMetaMethod::ToString.name())?;
                Ok(tostring.call(LuaValue::Table(table))?)
            } else {
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
            }
        },
        LuaValue::Thread(_) => Ok("#thread".to_string()),
        LuaValue::UserData(data) => {
            let meta = data.get_metatable()?;
            if meta.contains(LuaMetaMethod::ToString)? {
                let tostring: LuaFunction = meta.get(LuaMetaMethod::ToString)?;
                Ok(tostring.call(LuaValue::UserData(data))?)
            } else { Ok("#userdata".to_string()) }
        },
        // LuaValue::Vector(x, y, z) => Ok(format!("{{x: {}, y: {}, z: {}}}", x, y, z)),
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