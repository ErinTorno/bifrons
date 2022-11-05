use bevy::{prelude::*};

use crate::data::{anim::SceneOverride};
use crate::system::common::ToInit;

#[derive(Clone, Debug, Default)]
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system(init_scene)
        ;
    }
}

fn init_scene(
    mut commands:  Commands,
    to_init_scenes: Query<(Entity, &SceneOverride), With<ToInit<Scene>>>,
    children: Query<&Children>,
    name_query: Query<&Name>,
) {
    for (scene_entity, overrides) in to_init_scenes.iter() {
        let mut is_init = false;
        iter_hierarchy(scene_entity, &children, &mut |entity| {
            if let Ok(name) = name_query.get(entity) {
                is_init = true;
                if let Some(handle) = overrides.mat_overrides.get(name.as_str()) {
                    commands.entity(entity).insert(handle.clone());

                } else {
                    info!("No override for {:?}", name);
                }
            }
        });
        if is_init {
            commands.entity(scene_entity).remove::<ToInit<Scene>>();
        }
    }
}

fn iter_hierarchy(entity: Entity, children_query: &Query<&Children>, f: &mut impl FnMut(Entity)) {
    (f)(entity);
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter().copied() {
            iter_hierarchy(child, children_query, f);
        }
    }
}