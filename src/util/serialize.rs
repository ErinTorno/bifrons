
use bevy::prelude::Color;
use serde::{Deserialize, Deserializer, de, Serializer, Serialize};

pub fn deserialize_into<'de, D, I, R>(d: D) -> Result<R, D::Error> where D: Deserializer<'de>, I: Into<R>, I: Deserialize<'de> {
    let intermediate: I = de::Deserialize::deserialize(d)?;
    Ok(intermediate.into())
}
pub fn serialize_into<S, I, R>(i: &I, ser: S) -> Result<S::Ok, S::Error> where S: Serializer, I: Into<R>, I: Clone, R: Serialize {
    let r: R = i.clone().into();
    Serialize::serialize(&r, ser)
}
pub fn deserialize_into_option<'de, D, I, R>(d: D) -> Result<Option<R>, D::Error> where D: Deserializer<'de>, I: Into<R>, I: Deserialize<'de> {
    match Option::<I>::deserialize(d)? {
        Some(intermediate) => Ok(Some(intermediate.into())),
        None => Ok(None),
    }
}
pub fn serialize_into_option<S, I, R>(i: &Option<I>, ser: S) -> Result<S::Ok, S::Error> where S: Serializer, I: Into<R>, I: Clone, R: Serialize {
    let v: Option<R> = i.clone().map(|a| a.into());
    Serialize::serialize(&v, ser)
}

pub fn deserialize_hex_color<'de, D>(d: D) -> Result<Color, D::Error> where D: Deserializer<'de> {
    let full: String = de::Deserialize::deserialize(d)?;
    let s = if full.starts_with('#') { &full[1..] } else { full.as_str() };
    Color::hex(s).map_err(|e| de::Error::custom(e))
}
pub fn serialize_hex_color<S>(color: &Color, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let c = color.clone().as_rgba_f32();
    let s = format!("#{:02X}{:02X}{:02X}{:02X}", (c[0] * 255.) as u8, (c[1] * 255.) as u8, (c[2] * 255.) as u8, (c[3] * 255.) as u8);
    Serialize::serialize(&s, ser)
}