use bevy::prelude::Color;
use ron::{Options, extensions::Extensions};
use serde::{Deserializer, de, Serializer, Serialize};

use super::IntoHex;

pub fn ron_options() -> Options {
    Options::default().with_default_extension(Extensions::all())
}