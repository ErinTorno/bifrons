use bevy::{asset::*, prelude::*, reflect::TypeUuid, utils::HashSet};
use serde::{Deserialize, Serialize};

use super::anim::Animation;

#[derive(Clone, Debug, Deserialize, Serialize, TypeUuid)]
#[uuid = "68fbd47c-252c-409d-94f0-f581051ca8a5"]
pub struct Actor {
    #[serde(default)]
    pub tags:      HashSet<String>,
    pub animation: Animation,
}

#[derive(Clone, Component, Debug, Default)]
pub struct ToSpawnActor {
    pub handle: Handle<Actor>,
}

#[derive(Default)]
pub struct ActorLoader;

impl AssetLoader for ActorLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let actor: Actor = ron::de::from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(actor));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["actor.ron"]
    }
}