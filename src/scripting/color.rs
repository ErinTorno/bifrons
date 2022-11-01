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
        fields.add_field_method_set("hue", |_, this, hue| {
            let color: Color = this.clone().into();
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
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
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