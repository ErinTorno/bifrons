use std::hash::{Hash, Hasher};

use bevy::{prelude::{ClearColor, Color}, reflect::{Reflect, FromReflect}};
use bevy_inspector_egui::Inspectable;
use mlua::prelude::*;
use palette::*;
use serde::{de, Serialize, Deserialize, Deserializer, Serializer};

use crate::{util::IntoHex, data::lua::LuaWorld};

use super::LuaMod;

#[derive(Clone, Copy, Debug, Default, FromReflect, Inspectable, PartialEq, Reflect)]
pub struct RgbaColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl RgbaColor {
    pub const BLACK:   RgbaColor = RgbaColor {r: 0., g: 0., b: 0., a: 1.};
    pub const WHITE:   RgbaColor = RgbaColor {r: 1., g: 1., b: 1., a: 1.};
    pub const FUSCHIA: RgbaColor = RgbaColor {r: 0.56, g: 0.34, b: 0.64, a: 1.};
}
impl Eq for RgbaColor {}
impl Hash for RgbaColor {
    fn hash<H>(&self, hasher: &mut H) where H: Hasher {
        let safe_to_bits = |f: f32| if f.is_nan() {
            f32::NAN
        } else {
            (f * 100000.0).round() / 100000.0
        }.to_bits();
        hasher.write_u32(safe_to_bits(self.r));
        hasher.write_u32(safe_to_bits(self.g));
        hasher.write_u32(safe_to_bits(self.b));
        hasher.write_u32(safe_to_bits(self.a));
    }
}
impl<'de> Deserialize<'de> for RgbaColor {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s: String = Deserialize::deserialize(d)?;
        Color::hex(&s[1..].to_string())
            .map(RgbaColor::from)
            .map_err(|e| de::Error::custom(format!("{}", e)))
    }
}
impl Serialize for RgbaColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let s = format!("#{}", Color::from(*self).into_hex());
        serializer.serialize_str(s.as_str())
    }
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
    fn mod_name() -> &'static str { "Rgba" }
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
        table.set("white", RgbaColor::from(Color::WHITE))?;
        Ok(())
    }
}
impl From<Color> for RgbaColor {
    fn from(color: Color) -> Self {
        let c = color.as_rgba_f32();
        RgbaColor { r: c[0], g: c[1], b: c[2], a: c[3] }
    }
}
impl From<RgbaColor> for Color {
    fn from(c: RgbaColor) -> Color {
        Color::Rgba { red: c.r, green: c.g, blue: c.b, alpha: c.a }
    }
}
impl IntoHex for RgbaColor {
    fn into_hex(&self) -> String {
        Color::from(self.clone()).into_hex()
    }
}
// impl Into<RgbaColor> for Lcha {
//     fn into(self) -> RgbaColor {
//         let rgba: Rgba = Rgba::new(self.r, self.g, self.b, self.a);
//         Lcha::from_color(rgba)
//     }
// }
impl Clamp for RgbaColor {
    fn clamp(&self) -> Self {
        RgbaColor { r: self.r.clamp(0., 1.), g: self.g.clamp(0., 1.), b: self.b.clamp(0., 1.), a: self.a.clamp(0., 1.) }
    }

    fn is_within_bounds(&self) -> bool {
        self.r >= 0. && self.r <= 1. &&
        self.g >= 0. && self.g <= 1. &&
        self.b >= 0. && self.b <= 1. &&
        self.a >= 0. && self.a <= 1.
    }
    
    fn clamp_self(&mut self) {
        self.r = self.r.clamp(0., 1.);
        self.g = self.g.clamp(0., 1.);
        self.b = self.b.clamp(0., 1.);
        self.a = self.a.clamp(0., 1.);
    }
}

// #[derive(Copy, Clone, Debug, Eq, PartialEq)]
// pub struct LuaOklab(pub Oklaba);

