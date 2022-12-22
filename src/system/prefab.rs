use bevy::{prelude::*, asset::LoadState};
use bevy_inspector_egui::prelude::*;
use crate::{system::common::ToInit, data::{prefab::*, input::{ActionState, InputMap}, material::{TexMatInfo, LoadedMaterials, MaterialColors, MaterialsToInit}, stat::Attributes, lua::{LuaScriptVars}}, util::pair_clone};

use super::{texture::{Background}, camera::{ActiveCamera, Focus}, common::ToInitHandle, lua::{ToInitScripts, SharedInstances, LuaQueue}};

#[derive(Clone, Debug, Default)]
pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_startup_system(temp_setup)
            .add_system(spawn_prefab)
        ;
    }
}

#[derive(Clone, Component, Copy, Debug, Inspectable)]
pub struct Player {
    pub id: u32,
}

pub fn temp_setup(
    mut commands: Commands,
    asset_server:  Res<AssetServer>,
) {
    commands.spawn((
        LuaScriptVars::default(),
        Name::new("player"),
        Player { id: 0 },
        ToInitHandle::<Prefab>::new(asset_server.load("chars/labolas.prefab.ron")),
        TransformBundle {
            ..default()
        },
        VisibilityBundle::default(),
    ));
}

pub fn spawn_prefab(
    mut commands:      Commands,
    mut mat_colors:    ResMut<MaterialColors>,
    mut mats_to_init:  ResMut<MaterialsToInit>,
    asset_server:      Res<AssetServer>,
    background:        Res<Background>,
    mut lua_instances: ResMut<SharedInstances>,
    mut tex_mat_info:  ResMut<TexMatInfo>,
    prefabs:           Res<Assets<Prefab>>,
    mut meshes:        ResMut<Assets<Mesh>>,
    mut materials:     ResMut<Assets<StandardMaterial>>,
    mut to_spawn:      Query<(Entity, &ToInitHandle<Prefab>, &mut LuaScriptVars, Option<&Player>, Option<&LoadedMaterials>, Option<&mut Attributes>)>,
) {
    for (entity, ToInitHandle(handle), mut script_vars, player, loaded_mats, attributes) in to_spawn.iter_mut() {
        if let Some(prefab) = prefabs.get(&handle) {
            let entity = commands.entity(entity)
                .insert((
                    ActionState::default(),
                    InputMap::default(),
                ))
                .remove::<ToInitHandle<Prefab>>()
                .id();
            let mut loaded = prefab.animation.add_parts(entity, &mut commands, mat_colors.as_mut(), &mut mats_to_init.as_mut(), &asset_server, &mut tex_mat_info, &background, meshes.as_mut(), materials.as_mut());
            if let Some(mats) = loaded_mats {
                loaded.by_name.extend((&mats.by_name).into_iter().map(|(k, v)| (k.clone(), v.clone())));
            }
            
            commands.entity(entity).insert(loaded);

            if !prefab.tags.is_empty() {
                commands.entity(entity)
                    .insert(Tags(prefab.tags.clone()));
            }

            if !prefab.scripts.is_empty() {
                script_vars.merge(&prefab.script_vars);

                commands.entity(entity)
                    .insert((
                        LuaQueue::default(),
                        ToInitScripts {
                            handles: prefab.scripts.iter().map(|s| (lua_instances.gen_next_id(), asset_server.load(s))).collect(),
                        },
                    ));
            }
            if let Some(attr) = &prefab.attributes {
                if let Some(mut attributes) = attributes {
                    attributes.pools.extend(attr.pools.iter().map(pair_clone));
                    attributes.stats.extend(attr.stats.iter().map(pair_clone));
                } else {
                    commands.entity(entity)
                        .insert(attr.clone());
                }
            }
            if player.is_some() {
                commands.spawn((
                    ToInit::<Camera3d>::default(),
                    ActiveCamera {
                        controller: Some(entity),
                        focus: Focus::Entity { which: entity, offset: Vec3::ZERO }
                    },
                ));
            }
        } else {
            match asset_server.get_load_state(handle) {
                LoadState::Failed => {
                    let path = asset_server.get_handle_path(handle);
                    error!("Prefab asset failed for {:?} {:?}", entity, path);
                    commands.entity(entity).remove::<ToInitHandle<Prefab>>();
                },
                _ => (),
            }
        }
    }
}