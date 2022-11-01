use ::std::sync::Mutex;

use bevy::prelude::{info, ClearColor, Color};
use bevy_mod_scripting::{prelude::*, lua::api::bevy::LuaWorld};
use mlua::Lua;

#[derive(Default)]
pub struct ColorAPIProvider;

impl APIProvider for ColorAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_color_lua(ctx).map_err(ScriptError::new_other)?;
        info!("finished attaching ColorAPIProvider");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RgbaColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl<'lua> ToLua<'lua> for RgbaColor {
    fn to_lua(self, lua: &'lua Lua) -> Result<LuaValue<'lua>, LuaError> {
        let table = lua.create_table()?;
        table.set("r", self.r)?;
        table.set("g", self.g)?;
        table.set("b", self.b)?;
        table.set("a", self.a)?;
        table.to_lua(lua)
    }
}
impl<'lua> FromLua<'lua> for RgbaColor {
    fn from_lua(value: mlua::Value<'lua>, _: &'lua Lua) -> Result<Self, mlua::Error> {
        match value {
            Value::Table(table) => {
                let r = table.get("r")?;
                let g = table.get("g")?;
                let b = table.get("b")?;
                let a = table.get("a").unwrap_or(1.);
                Ok(RgbaColor { r, g, b, a })
            }
            _ => Err(mlua::Error::SerializeError(format!("Expected {{r: num[0..1], g: num[0..1], b: num[0..1], a: num[0..1]}}, found {:?}", value))),
        }
    }
}
impl From<Color> for RgbaColor {
    fn from(color: Color) -> Self {
        let c = color.as_rgba_f32();
        RgbaColor { r: c[0], g: c[1], b: c[2], a: c[3] }
    }
}
impl Into<Color> for RgbaColor {
    fn into(self) -> Color {
        Color::Rgba { red: self.r, green: self.g, blue: self.b, alpha: self.a }
    }
}

fn attach_color_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    let table = ctx.create_table()?;
    table.set("hex", ctx.create_function(|_, hex: String| {
            let s = if hex.starts_with('#') { &hex[1..] } else { hex.as_str() };
            Color::hex(s)
                .map_err(|h| mlua::Error::DeserializeError(h.to_string()))
                .map(RgbaColor::from)
        })?
    )?;
    table.set("background", ctx.create_function(|ctx, ()| {
            Ok(ctx.globals().get::<_, LuaWorld>("world").unwrap().read()
                .get_resource::<ClearColor>().map(|c| Into::<RgbaColor>::into(c.0)))
        })?
    )?;
    table.set("set_background", ctx.create_function(|ctx, rgba: RgbaColor| {
            let color = rgba.into();
            ctx.globals().get::<_, LuaWorld>("world").unwrap().write()
                .insert_resource(ClearColor(color));
            Ok(())
        })?
    )?;
    ctx.globals().set("Color", table)?;
    Ok(())
}