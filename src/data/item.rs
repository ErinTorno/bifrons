use bevy::{asset::{LoadContext, AssetLoader, LoadedAsset}, utils::BoxedFuture, reflect::TypeUuid};
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::scripting::LuaScriptVars;

use super::{lang::Lines, anim::Animation};


#[derive(Clone, Debug, Deserialize, Serialize, TypeUuid)]
#[uuid = "207e1deb-5abb-4892-8e99-b740a2f7ae61"]
pub struct Item {
    #[serde(default)]
    pub scripts:     Vec<String>,
    #[serde(default)]
    pub script_vars: LuaScriptVars,
    #[serde(default)]
    pub lines:       Lines,
    #[serde(default)]
    pub tags:        HashSet<String>,
    #[serde(default)]
    pub equip_slots: HashSet<String>,
    pub animation:   Animation,
}

#[derive(Default)]
pub struct ItemLoader;

impl AssetLoader for ItemLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let item: Item = ron::de::from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(item));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["item.ron"]
    }
}