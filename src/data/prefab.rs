use bevy::{asset::*, prelude::*, reflect::TypeUuid, utils::HashSet};
use serde::{Deserialize, Serialize};

use crate::scripting::LuaScriptVars;

use super::{anim::Animation, lang::Lines};

#[derive(Clone, Debug, Deserialize, Serialize, TypeUuid)]
#[uuid = "68fbd47c-252c-409d-94f0-f581051ca8a5"]
pub struct Prefab {
    #[serde(default)]
    pub scripts:   Vec<String>,
    #[serde(default)]
    pub script_vars: LuaScriptVars,
    #[serde(default)]
    pub tags:      HashSet<String>,
    #[serde(default)]
    pub lines:     Lines,
    pub animation: Animation,
}

#[derive(Default)]
pub struct PrefabLoader;

impl AssetLoader for PrefabLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let prefab: Prefab = ron::de::from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(prefab));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["prefab.ron"]
    }
}