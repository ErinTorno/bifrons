
use bevy::prelude::Color;
use serde::{Deserializer, de, Serializer, Serialize};

use super::IntoHex;

pub fn deserialize_hex_color<'de, D>(d: D) -> Result<Color, D::Error> where D: Deserializer<'de> {
    let full: String = de::Deserialize::deserialize(d)?;
    let s = if full.starts_with('#') { &full[1..] } else { full.as_str() };
    Color::hex(s).map_err(|e| de::Error::custom(e))
}
pub fn serialize_hex_color<S>(color: &Color, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
    Serialize::serialize(&color.into_hex(), ser)
}