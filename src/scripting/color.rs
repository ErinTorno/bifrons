use ::std::sync::Mutex;

use bevy::prelude::{ClearColor, Color};
use bevy_mod_scripting::{prelude::*, lua::api::bevy::LuaWorld};
use mlua::Lua;

use crate::{util::IntoHex, scripting::init_luamod};

use super::LuaMod;

#[derive(Default)]
pub struct ColorAPIProvider;

impl APIProvider for ColorAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        init_luamod::<RgbaColor>(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbaColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl LuaUserData for RgbaColor {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("r", |_, this| Ok(this.r));
        fields.add_field_method_get("g", |_, this| Ok(this.g));
        fields.add_field_method_get("b", |_, this| Ok(this.b));
        fields.add_field_method_get("a", |_, this| Ok(this.a));
        fields.add_field_method_set("r", |_, this, r| { this.r = r; Ok(()) });
        fields.add_field_method_set("g", |_, this, g| { this.g = g; Ok(()) });
        fields.add_field_method_set("b", |_, this, b| { this.b = b; Ok(()) });
        fields.add_field_method_set("a", |_, this, a| { this.a = a; Ok(()) });
        fields.add_field_method_get("hue", |_, this| Ok(<RgbaColor as Into<Color>>::into(this.clone()).as_hsla_f32()[0]) );
        fields.add_field_method_set("hue", |_, this, hue: f32| {
            let color: Color = this.clone().into();
            let hue = hue % 360.;
            let hue = if hue < 0. { hue + 360. } else { hue };
            if let Color::Hsla { hue: _, saturation, lightness, alpha } = color.as_hsla() {
                *this = Color::hsla(hue, saturation, lightness, alpha).into();
            } else { unreachable!() }
            Ok(())
        });
        fields.add_field_method_get("saturation", |_, this| Ok(<RgbaColor as Into<Color>>::into(this.clone()).as_hsla_f32()[1]) );
        fields.add_field_method_set("saturation", |_, this, saturation| {
            let color: Color = this.clone().into();
            if let Color::Hsla { hue, saturation: _, lightness, alpha } = color.as_hsla() {
                *this = Color::hsla(hue, saturation, lightness, alpha).into();
            } else { unreachable!() }
            Ok(())
        });
        fields.add_field_method_get("lightness", |_, this| Ok(<RgbaColor as Into<Color>>::into(this.clone()).as_hsla_f32()[2]) );
        fields.add_field_method_set("lightness", |_, this, lightness| {
            let color: Color = this.clone().into();
            if let Color::Hsla { hue, saturation, lightness: _, alpha } = color.as_hsla() {
                *this = Color::hsla(hue, saturation, lightness, alpha).into();
            } else { unreachable!() }
            Ok(())
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(<RgbaColor as Into<Color>>::into(this.clone()).into_hex()));
    }
}
impl LuaMod for RgbaColor {
    fn mod_name() -> &'static str { "Color" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("hex", lua.create_function(|_, hex: String| {
                let s = if hex.starts_with('#') { &hex[1..] } else { hex.as_str() };
                Color::hex(s)
                    .map_err(|h| mlua::Error::DeserializeError(h.to_string()))
                    .map(RgbaColor::from)
            })?
        )?;
        table.set("background", lua.create_function(|ctx, ()| {
                Ok(ctx.globals().get::<_, LuaWorld>("world").unwrap().read()
                    .get_resource::<ClearColor>().map(|c| Into::<RgbaColor>::into(c.0)))
            })?
        )?;
        table.set("set_background", lua.create_function(|ctx, rgba: RgbaColor| {
                let color = rgba.into();
                ctx.globals().get::<_, LuaWorld>("world").unwrap().write()
                    .insert_resource(ClearColor(color));
                Ok(())
            })?
        )?;
        table.set("black", RgbaColor::from(Color::BLACK))?;
        table.set("clear", RgbaColor { r: 0., g: 0., b: 0., a: 0.})?;
        table.set("white", RgbaColor::from(Color::WHITE))?;
        table.set("red", RgbaColor::from(Color::hex("8d3e29").unwrap()))?;
        table.set("green", RgbaColor::from(Color::hex("579035").unwrap()))?;
        table.set("blue", RgbaColor::from(Color::hex("5f97b6").unwrap()))?;
        table.set("bone", RgbaColor::from(Color::hex("deceb4").unwrap()))?;
        table.set("burgundy", RgbaColor::from(Color::hex("5e292f").unwrap()))?;
        table.set("icy", RgbaColor::from(Color::hex("9cabb1").unwrap()))?;
        table.set("orange", RgbaColor::from(Color::hex("e09b4d").unwrap()))?;
        table.set("sycamore", RgbaColor::from(Color::hex("799240").unwrap()))?;
        table.set("wine",   RgbaColor::from(Color::hex("4b4158").unwrap()))?;
        Ok(())
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