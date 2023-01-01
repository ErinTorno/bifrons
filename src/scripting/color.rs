use std::{hash::{Hash, Hasher}, str::FromStr};

use bevy::{prelude::{Color}, reflect::{Reflect, FromReflect}};
use bevy_egui::{egui};
use bevy_inspector_egui::Inspectable;
use mlua::prelude::*;
use palette::{*, convert::FromColorUnclamped, rgb::{Rgba}};
use serde::{de, Serialize, Deserialize, Deserializer, Serializer};

use crate::{util::{IntoHex, RoughlyEq}, data::lua::{Any2}};

use super::LuaMod;

#[derive(Clone, Copy, Debug, Default, FromReflect, Inspectable, PartialEq, Reflect)]
pub struct RgbaColor {
    pub is_linear: bool,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl RgbaColor {
    pub const BLACK:       RgbaColor = RgbaColor {r: 0., g: 0., b: 0., a: 1., is_linear: false};
    pub const TRANSPARENT: RgbaColor = RgbaColor {r: 0., g: 0., b: 0., a: 0., is_linear: false};
    pub const WHITE:       RgbaColor = RgbaColor {r: 1., g: 1., b: 1., a: 1., is_linear: false};
    pub const FUCHSIA:     RgbaColor = RgbaColor {r: 0.56, g: 0.34, b: 0.64, a: 1., is_linear: false};

    pub fn as_linear(&self) -> RgbaColor {
        if self.is_linear {
            self.clone()
        } else {
            if let Color::RgbaLinear { red, green, blue, alpha } = Color::from(self.clone()).as_rgba_linear() {
                RgbaColor { r: red, g: green, b: blue, a: alpha, is_linear: true }
            } else { unreachable!() }
        }
    }

    pub fn as_srgb(&self) -> RgbaColor {
        if self.is_linear {
            if let Color::Rgba { red, green, blue, alpha } = Color::from(self.clone()).as_rgba() {
                RgbaColor { r: red, g: green, b: blue, a: alpha, is_linear: false }
            } else { unreachable!() }
        } else {
            self.clone()
        }
    }

    pub fn linear_op<F>(&self, that: &RgbaColor, f: F) -> RgbaColor where F: Fn(f32, f32) -> f32 {
        let this = self.as_linear();
        let that = that.as_linear();
        let result = RgbaColor {
            r: f(this.r, that.r),
            g: f(this.g, that.g),
            b: f(this.b, that.b),
            a: f(this.a, that.a).min(1.).max(0.),
            is_linear: true,
        };
        if self.is_linear { result } else { result.as_srgb() } // convert back if self is sRGB
    }

    pub fn linear_f32_op<F>(&self, m: f32, f: F) -> RgbaColor where F: Fn(f32, f32) -> f32 {
        let this = self.as_linear();
        let result = RgbaColor {
            r: f(this.r, m),
            g: f(this.g, m),
            b: f(this.b, m),
            a: this.a,
            is_linear: true,
        };
        if self.is_linear { result } else { result.as_srgb() } // convert back if self is sRGB
    }

    pub fn lua_to_string(&self) -> String {
        format!(
            "{}{{r = {}, g = {}, b = {}, a = {}}}",
            if self.is_linear { "linear" } else { "srgb" },
            self.r,
            self.g,
            self.b,
            self.a,
        )
    }
}
impl Eq for RgbaColor {}
impl RoughlyEq<RgbaColor> for RgbaColor {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon { 0.0001 }

    fn roughly_eq_within(&self, that: &RgbaColor, epsilon: Self::Epsilon) -> bool {
        let this = self;
        let that = if self.is_linear { that.as_linear() } else { that.as_srgb() };
        this.r.roughly_eq_within(&that.r, epsilon) &&
        this.g.roughly_eq_within(&that.g, epsilon) &&
        this.b.roughly_eq_within(&that.b, epsilon) &&
        this.a.roughly_eq_within(&that.a, epsilon)
    }
}
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
impl FromStr for RgbaColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("#") {
            Color::hex(&s[1..].to_string())
                .map(RgbaColor::from)
                .map_err(|e| format!("{}", e))
        } else if s.starts_with("linear(") {
            #[derive(Deserialize)]
            struct Linear { r: f32, g: f32, b: f32, #[serde(default)] a: f32 }

            let linear: Linear = ron::de::from_str(&s[6..]).map_err(|e| format!("RgbaColor linear deserialization error {}", e))?;
            Ok(RgbaColor { is_linear: true, r: linear.r, g: linear.g, b: linear.b, a: linear.a })
        } else {
            Err(format!("Invalid rgba string: {}", s))
        }
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
        fields.add_field_method_set("r", |_, this, r| { this.r = r; Ok(()) });
        fields.add_field_method_get("g", |_, this| Ok(this.g));
        fields.add_field_method_set("g", |_, this, g| { this.g = g; Ok(()) });
        fields.add_field_method_get("b", |_, this| Ok(this.b));
        fields.add_field_method_set("b", |_, this, b| { this.b = b; Ok(()) });
        fields.add_field_method_get("a", |_, this| Ok(this.a));
        fields.add_field_method_set("a", |_, this, a| { this.a = a; Ok(()) });
        fields.add_field_method_get("is_linear", |_, this| Ok(this.is_linear));
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
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, that: RgbaColor| Ok(this.linear_op(&that, std::ops::Add::add)));
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, any: Any2<f32, RgbaColor>| Ok(match any {
            Any2::A(m) => this.linear_f32_op(m, std::ops::Div::div),
            Any2::B(that) => this.linear_op(&that, std::ops::Div::div),
        }));
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, any: Any2<f32, RgbaColor>| Ok(match any {
            Any2::A(m)    => this.linear_f32_op(m, std::ops::Mul::mul),
            Any2::B(that) => this.linear_op(&that, std::ops::Mul::mul),
        }));
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, that: RgbaColor| Ok(this.linear_op(&that, std::ops::Sub::sub)));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(<RgbaColor as Into<Color>>::into(this.clone()).into_hex()));

        methods.add_method("difference_from", |_, this, that: RgbaColor| Ok({
            let this = Lcha::from_color(this.clone());
            let that = Lcha::from_color(that.clone());
            this.get_color_difference(&that)
        }));
        methods.add_method("linear", |_, this, ()| Ok(this.as_linear()));
        methods.add_method("srgb", |_, this, ()| Ok(this.as_srgb()));
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: RgbaColor| Ok(this.roughly_eq_within(&that, 0.0000001)));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(this.lua_to_string()));
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
        })?)?;
        table.set("new", lua.create_function(|_, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            Ok(RgbaColor { r, g, b, a: a.unwrap_or(1.), is_linear: false })
        })?)?;
        table.set("new_linear", lua.create_function(|_, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
            Ok(RgbaColor { r, g, b, a: a.unwrap_or(1.), is_linear: false })
        })?)?;
        table.set("black", lua.create_function(|_, ()| Ok(RgbaColor::BLACK))?)?;
        table.set("fuchsia", lua.create_function(|_, ()| Ok(RgbaColor::FUCHSIA))?)?;
        table.set("white", lua.create_function(|_, ()| Ok(RgbaColor::WHITE))?)?;
        Ok(())
    }
}
impl From<Color> for RgbaColor {
    fn from(color: Color) -> Self {
        match color {
            Color::RgbaLinear { red, green, blue, alpha } => RgbaColor { r: red, g: green, b: blue, a: alpha, is_linear: true },
            _ => {
                let [r, g, b, a] = color.as_rgba_f32();
                RgbaColor { r, g, b, a, is_linear: false }
            },
        }
        
    }
}
impl From<RgbaColor> for Color {
    fn from(c: RgbaColor) -> Color {
        if c.is_linear {
            Color::RgbaLinear { red: c.r, green: c.g, blue: c.b, alpha: c.a }
        } else {
            Color::Rgba { red: c.r, green: c.g, blue: c.b, alpha: c.a }
        }
    }
}
impl From<RgbaColor> for egui::Color32 {
    fn from(value: RgbaColor) -> Self {
        egui::Rgba::from_rgba_premultiplied(value.r, value.g, value.b, value.a).into()
    }
}
impl IntoHex for RgbaColor {
    fn into_hex(&self) -> String {
        Color::from(self.clone()).into_hex()
    }
}
impl Clamp for RgbaColor {
    fn clamp(&self) -> Self {
        RgbaColor { r: self.r.clamp(0., 1.), g: self.g.clamp(0., 1.), b: self.b.clamp(0., 1.), a: self.a.clamp(0., 1.), is_linear: self.is_linear }
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
impl FromColorUnclamped<Color> for RgbaColor {
    fn from_color_unclamped(val: Color) -> Self {
        RgbaColor::from(val)
    }
}
impl FromColorUnclamped<Lcha> for RgbaColor {
    fn from_color_unclamped(val: Lcha) -> Self {
        let val: Rgba = Rgba::from_color(val);
        RgbaColor { r: val.red, g: val.green, b: val.blue, a: val.alpha, is_linear: false }
    }
}
impl FromColorUnclamped<Oklcha> for RgbaColor {
    fn from_color_unclamped(val: Oklcha) -> Self {
        let val: Rgba = Rgba::from_color(val);
        RgbaColor { r: val.red, g: val.green, b: val.blue, a: val.alpha, is_linear: false }
    }
}
impl FromColorUnclamped<Rgba> for RgbaColor {
    fn from_color_unclamped(val: Rgba) -> Self {
        RgbaColor { r: val.red, g: val.green, b: val.blue, a: val.alpha, is_linear: false }
    }
}
impl FromColorUnclamped<RgbaColor> for Color {
    fn from_color_unclamped(val: RgbaColor) -> Self {
        val.into()
    }
}
impl FromColorUnclamped<RgbaColor> for Lcha {
    fn from_color_unclamped(val: RgbaColor) -> Self {
        let rgba: Rgba = Rgba::from_color(val);
        Lcha::from_color(rgba)
    }
}
impl FromColorUnclamped<RgbaColor> for Oklcha {
    fn from_color_unclamped(val: RgbaColor) -> Self {
        let rgba: Rgba = Rgba::from_color(val);
        Oklcha::from_color(rgba)
    }
}
impl FromColorUnclamped<RgbaColor> for Rgba {
    fn from_color_unclamped(val: RgbaColor) -> Self {
        Rgba::new(val.r, val.g, val.b, val.a)
    }
}