use std::collections::HashMap;

use bevy::{prelude::*, utils::BoxedFuture, asset::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

use super::{geometry::{Geometry, Light}, material::TextureMaterial, grid::CellID};
use crate::{util::serialize::*, scripting::LuaScriptVars};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PrefabLocation {
    Free(Vec3),
    Cell(CellID),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrefabInstance {
    #[serde(default)]
    pub label: Option<String>,
    pub asset: String,
    pub at:    PrefabLocation,
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(default = "default_room_child")]
    pub room_child: bool,
    #[serde(default)]
    pub script_vars: LuaScriptVars,
}
fn default_room_child() -> bool { true }

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Room {
    #[serde(default)]
    pub reveal_before_entry: bool,
    pub pos:      Vec3,
    #[serde(default)]
    pub keep_loaded: bool,
    #[serde(default)]
    pub prefabs: Vec<PrefabInstance>,
    #[serde(default)]
    pub geometry: Vec<Geometry>,
    #[serde(default)]
    pub lights:   Vec<Light>,
}

#[derive(Clone, Component, Debug, Default, Deserialize, Serialize)]
pub struct InRoom {
    pub room: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, TypeUuid)]
#[uuid = "a491e648-a317-40e9-a1eb-69f4532f2258"]
pub struct Level {
    pub name:       String,
    #[serde(default = "default_scripts")]
    pub scripts:    Vec<String>,
    #[serde(default = "default_background", deserialize_with = "deserialize_hex_color", serialize_with = "serialize_hex_color")]
    pub background: Color,
    pub materials:  HashMap<String, TextureMaterial>,
    pub rooms:      HashMap<String, Room>,
}
pub fn default_background() -> Color { Color::BLACK }
pub fn default_scripts() -> Vec<String> { Vec::new() }

#[derive(Default)]
pub struct LevelLoader;

impl AssetLoader for LevelLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let actor: Level = ron::de::from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(actor));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level.ron"]
    }
}