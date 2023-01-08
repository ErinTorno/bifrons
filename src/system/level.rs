use std::{collections::{HashMap, HashSet}};

use bevy::{prelude::*};
use indexmap::IndexMap;

use crate::{data::{level::*, material::{TextureMaterial, AtlasIndex, TexMatInfo, MaterialColors, MaterialsToInit, LoadedMat}, geometry::{Shape, LightAnimState, LightAnim}, prefab::{PrefabLoader, Prefab}, lua::{LuaScript, Hook, ManyTransVars, TransVar, LuaTransVars}}, scripting::{event::{ON_ROOM_REVEAL, EventFlag}}};

use super::{texture::{MissingTexture, Background}, common::{fix_missing_extension, ToInitHandle}, lua::{ToInitScripts, SharedInstances, LuaQueue, HookCall, LuaEventQueue, EventCall}};

#[derive(Clone, Debug, Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_asset::<Level>()
            .add_asset::<LoadedLevel>()
            .init_asset_loader::<LevelLoader>()
            .init_resource::<LoadedLevelCache>()
            .add_system(load_levels)
            .add_system(spawn_level)
            .add_system(spawn_room)
            .register_type::<LightAnim>()
            .register_type::<LightAnimState>()
        ;
    }
}

fn load_levels(
    mut events:        EventReader<AssetEvent<Level>>,
    asset_server:      Res<AssetServer>,
    levels:            Res<Assets<Level>>,
    mut ll_cache:      ResMut<LoadedLevelCache>,
    mut loaded_levels: ResMut<Assets<LoadedLevel>>,
    mut lua_instances: ResMut<SharedInstances>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(level) = levels.get(handle) {
                    let scripts: IndexMap<u32, Handle<LuaScript>> = level.scripts.iter()
                        .map(|s| (lua_instances.gen_next_id(), asset_server.load(s)))
                        .collect();

                    let loaded_level = LoadedLevel {
                        this_handle:  Handle::default(),
                        level_handle: Some(handle.clone_weak()),
                        materials:    level.materials.clone(),
                        rooms:        level.rooms.clone(),
                        scripts,
                        script_vars:  level.script_vars.0.iter()
                            .map(|(k, v)| (k.clone(), TransVar::from(v.clone())))
                            .collect(),
                    };
                    let this_handle    = loaded_levels.add(loaded_level);

                    let mut loaded     = loaded_levels.get_mut(&this_handle).unwrap();
                    loaded.this_handle = this_handle.clone_weak();

                    ll_cache.loaded_by_level.insert(handle.clone_weak(), this_handle);
                }
            },
            _ => (),
        }
    }
}

fn spawn_level(
    mut commands:      Commands,
    asset_server:      Res<AssetServer>,
    loaded_levels:     Res<Assets<LoadedLevel>>,
    mut materials:     ResMut<Assets<StandardMaterial>>,
    mut mat_colors:    ResMut<MaterialColors>,
    mut mats_to_init:  ResMut<MaterialsToInit>,
    mut tex_mat_info:  ResMut<TexMatInfo>,
    query:             Query<(Entity, Option<&Visibility>, &ToInitHandle<LoadedLevel>)>,
) {
    for (entity, visibility, ToInitHandle(to_init)) in query.iter() {
        if let Some(level) = loaded_levels.get(to_init) {
            let waiting_scripts: HashSet<u32> = level.scripts.keys().cloned().collect();
            
            commands.entity(entity)
                .remove::<ToInitHandle<LoadedLevel>>()
                .insert((
                    LuaTransVars(level.script_vars.clone()),
                    LuaQueue::default(),
                    ToInitScripts { handles: level.scripts.clone() },
                )).add_children(|parent| {
                    let materials: HashMap<String, (Handle<StandardMaterial>, TextureMaterial)> = level.materials.iter()
                        .map(|(name, mat)| (name.clone(), (mat.load_material(&asset_server, tex_mat_info.as_mut(), materials.as_mut()), mat.clone())))
                        .collect();

                    for (handle, tex_mat) in materials.values() {
                        mat_colors.by_handle.insert(handle.clone_weak(), LoadedMat {
                            handle: handle.clone_weak(),
                            tex_mat: tex_mat.clone(),
                        });
                        mats_to_init.0.insert(handle.clone_weak());
                    }

                    for (room_name, room) in level.rooms.iter() {
                        parent.spawn(ToSpawnRoom {
                            materials: materials.clone(),
                            waiting_scripts: waiting_scripts.clone(),
                            room: room.clone(),
                            room_name: room_name.clone(),
                            is_revealed: visibility.map(|v| v.is_visible).unwrap_or(room.reveal_before_entry),
                        });
                    }
                });
        }
    }
}

// Room

#[derive(Clone, Component, Debug)]
pub struct ToSpawnRoom {
    materials:       HashMap<String, (Handle<StandardMaterial>, TextureMaterial)>,
    room:            Room,
    room_name:       String,
    is_revealed:     bool,
    waiting_scripts: HashSet<u32>,
}

pub fn spawn_room(
    mut commands:     Commands,
    missing_tex:      Res<MissingTexture>,
    background:       Res<Background>,
    asset_server:     Res<AssetServer>,
    mut meshes:       ResMut<Assets<Mesh>>,
    mut event_queue:  ResMut<LuaEventQueue>,
    query:            Query<(Entity, &ToSpawnRoom)>,
) {
    let background_texmat = TextureMaterial::BACKGROUND;
    let missing_texmat    = TextureMaterial::MISSING;

    for (room_entity, ToSpawnRoom { materials, room, room_name, waiting_scripts, is_revealed }) in query.iter() {
        let is_revealed = *is_revealed || room.reveal_before_entry;
        commands.entity(room_entity)
            .remove::<ToSpawnRoom>()
            .insert(VisibilityBundle {
                visibility: Visibility { is_visible: is_revealed },
                ..VisibilityBundle::default()
            })
            .insert(Name::from(room_name.clone()))
            .insert(TransformBundle {
                local: Transform::from_translation(room.pos),
                ..TransformBundle::default()
            })
            .add_children(|parent| {
                for geometry in room.geometry.iter() {
                    let mut layer_offset = 0.;
                    parent.spawn((
                        Name::new(geometry.label.as_ref().cloned().unwrap_or(format!("unnamed {}", geometry.shape.name()))),
                        TransformBundle {
                            local: Transform::from_translation(geometry.pos).with_rotation(
                                Quat::from_rotation_x(geometry.rotation.x) *
                                Quat::from_rotation_y(geometry.rotation.y) *
                                Quat::from_rotation_z(geometry.rotation.z)
                            ),
                            ..TransformBundle::default()
                        },
                        VisibilityBundle::default(),
                    )).add_children(|parent| {
                            for texname in geometry.materials.iter() {
                                let (material, texmat) = if texname == "background" {
                                    (background.material.clone(), &background_texmat)
                                } else if let Some((h, texmat)) = materials.get(texname) {
                                    (h.clone(), texmat)
                                } else {
                                    warn!("room {} was unable to find material {}; using default", room_name, texname);
                                    (missing_tex.material.clone(), &missing_texmat)
                                };
                                let mesh = meshes.add((match geometry.shape.clone() {
                                    Shape::Quad { w, h, d, one_sided } => Shape::Quad { w, h, d: d + layer_offset, one_sided },
                                    s => s,
                                }).mk_mesh(&texmat, Vec3::ZERO, AtlasIndex::default()));
                                parent.spawn((
                                    InRoom { room: room_name.clone() },
                                    PbrBundle { mesh, material, ..default() },
                                ));
                                layer_offset += 0.0001;
                            }
                        });
                }

                for light in room.lights.iter() {
                    let mut light_builder = parent.spawn_empty();
                    light.insert(&mut light_builder, Vec3::ZERO);
                }

                for prefab in room.prefabs.iter() {
                    if prefab.room_child {
                        let path = fix_missing_extension::<PrefabLoader>(prefab.asset.clone());
                        let mut ent = parent.spawn((
                            Name::new(prefab.label.as_ref().cloned().unwrap_or(format!("unnamed {}", prefab.asset))),
                            LuaTransVars::from(prefab.script_vars.clone()),
                            ToInitHandle::<Prefab>::new(asset_server.load(&path)),
                            TransformBundle {
                                local: Transform::from_translation(match prefab.at {
                                    PrefabLocation::Free(v) => v,
                                }).with_rotation(Quat::from_euler(EulerRot::XYZ, prefab.rotation.x, prefab.rotation.y, prefab.rotation.z)),
                                ..default()
                            },
                            VisibilityBundle::default(),
                        ));
                        if prefab.attributes.is_some() {
                            ent.insert(prefab.attributes.as_ref().unwrap().clone());
                        }
                    } else {
                        todo!();
                    }
                }
            });

        if is_revealed {
            let args = ManyTransVars(vec![
                room_name.clone().into(),
                room_entity.clone().into(),
            ]);
            commands.entity(room_entity).insert(LuaQueue {
                calls: vec![HookCall {
                    script_ids: waiting_scripts.clone(),
                    hook: Hook {
                        name: ON_ROOM_REVEAL.into(),
                        args: args.clone(),
                    },
                }]
            });
            event_queue.calls.push(
                EventCall {
                    flag: EventFlag::ON_ROOM_REVEAL,
                    hook: Hook {
                        name: ON_ROOM_REVEAL.into(),
                        args,
                    },
                }
            );
        }
    }
}