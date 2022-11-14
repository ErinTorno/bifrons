use std::{collections::{HashMap, HashSet}};

use bevy::{prelude::*};
use bevy_mod_scripting::{prelude::*};
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::{data::{level::*, material::{TextureMaterial, AtlasIndex, TexMatInfo}, geometry::Shape, prefab::{PrefabLoader, Prefab}}, scripting::{random, AwaitScript, event::ON_ROOM_REVEAL, ScriptVar}, util::InsertableWithPredicate};

use super::{texture::{MissingTexture, Background}, common::{fix_missing_extension, ToInitHandle}, editor::AssetInfo};

#[derive(Clone, Debug, Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .init_resource::<LoadingLevel>()
            .add_startup_system(startup)
            .add_system(spawn_level_piece)
            .add_system(spawn_room)
            .add_system(load_level
                .run_if_resource_exists::<LoadingLevel>())
            .add_system(reset_loaded_level
                .run_if_resource_exists::<LoadedLevel>());
    }
}

#[derive(Default)]
pub struct LoadingLevel {
    pub asset_info: Option<AssetInfo>,
    pub handle: Handle<Level>,
}

pub fn startup(mut st: ResMut<LoadingLevel>, asset_server: Res<AssetServer>) {
    let path      = "levels/testing/testing_house.level.ron";
    st.handle     = asset_server.load(path);
    st.asset_info = Some(AssetInfo::new(path));
}

pub fn load_level(
    mut st:        ResMut<LoadingLevel>,
    mut commands:  Commands,
    levels:        Res<Assets<Level>>,
) {
    if let Some(level) = levels.get(&st.handle) {
        let level_entity = commands.spawn()
            .insert_bundle(VisibilityBundle::default())
            .insert_bundle(TransformBundle::default())
            .id();
        if let Some(info) = st.asset_info.take() {
            commands.spawn().insert(info);
        }
        commands.remove_resource::<LoadingLevel>();
        commands
            .insert_resource(LoadedLevel {
                should_reset: true,
                level:        level.clone(),
                level_entity,
            });
    }
}

pub struct LoadedLevel {
    pub should_reset: bool,
    pub level:        Level,
    pub level_entity: Entity,
}

pub fn reset_loaded_level(
    mut st:           ResMut<LoadedLevel>,
    mut commands:     Commands,
    background:       Res<Background>,
    mut ccolor:       ResMut<ClearColor>,
    mut tex_mat_info: ResMut<TexMatInfo>,
    asset_server:     Res<AssetServer>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
) {
    if st.should_reset {
        let level = &st.level;
        random::set_seed(rand::prelude::random()); // todo should this be lobby setting?
        ccolor.0 = level.background;
        {
            let mut mat = materials.get_mut(&background.material).unwrap();
            mat.base_color = level.background;
        }

        // Scripts

        let scripts: Vec<Script<LuaFile>> = level.scripts.iter().map(|path| {
            let handle = asset_server.load::<LuaFile, _>(path);
            Script::<LuaFile>::new(path.clone(), handle)
        }).collect();

        let waiting_scripts: HashSet<u32> = scripts.iter().map(Script::id).collect();
        
        commands.entity(st.level_entity)
            .remove::<ScriptCollection::<LuaFile>>()
            .insert(Name::from(level.name.clone()))
            .insert(level.script_vars.clone())
            .insert(ScriptCollection::<LuaFile> { scripts });

        // Visuals

        let materials: HashMap<String, (Handle<StandardMaterial>, TextureMaterial)> = level.materials.iter()
            .map(|(name, mat)| (name.clone(), (mat.load_material(&asset_server, tex_mat_info.as_mut(), materials.as_mut()), mat.clone())))
            .collect();
        
        commands.entity(st.level_entity)
            .add_children(|parent| {
                for (room_name, room) in level.rooms.iter() {
                    parent.spawn().insert(ToSpawnRoom {
                        materials: materials.clone(),
                        waiting_scripts: waiting_scripts.clone(),
                        room: room.clone(),
                        room_name: room_name.clone(),
                        is_revealed: room.reveal_before_entry,
                    });
                }
            });
        st.should_reset = false;
    }
}

fn spawn_level_piece(
    mut commands:     Commands,
    mut tex_mat_info: ResMut<TexMatInfo>,
    asset_server:     Res<AssetServer>,
    level_pieces:     Res<Assets<LevelPiece>>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Name, &Visibility, &ToInitHandle<LevelPiece>)>,
) {
    for (entity, name, visibility, ToInitHandle(to_init)) in query.iter() {
        if let Some(level_piece) = level_pieces.get(to_init) {
            let scripts: Vec<Script<LuaFile>> = level_piece.scripts.iter().map(|path| {
                let handle = asset_server.load::<LuaFile, _>(path);
                Script::<LuaFile>::new(path.clone(), handle)
            }).collect();

            let waiting_scripts: HashSet<u32> = scripts.iter().map(Script::id).collect();

            commands.entity(entity)
                .remove::<ToInitHandle<LevelPiece>>()
                .remove::<ScriptCollection::<LuaFile>>()
                .insert(ScriptCollection::<LuaFile> { scripts })
                .insert(level_piece.script_vars.clone())
                .add_children(|parent| {
                    let materials: HashMap<String, (Handle<StandardMaterial>, TextureMaterial)> = level_piece.materials.iter()
                        .map(|(name, mat)| (name.clone(), (mat.load_material(&asset_server, tex_mat_info.as_mut(), materials.as_mut()), mat.clone())))
                        .collect();

                    parent.spawn().insert(ToSpawnRoom {
                        materials,
                        waiting_scripts,
                        room: level_piece.room.clone(),
                        room_name: name.to_string(),
                        is_revealed: visibility.is_visible,
                    });
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
    query:            Query<(Entity, &ToSpawnRoom)>,
) {
    let background_texmat = TextureMaterial::BACKGROUND;
    let missing_texmat    = TextureMaterial::MISSING;

    for (room_entity, ToSpawnRoom { materials, room, room_name, waiting_scripts, is_revealed }) in query.iter() {
        let is_revealed = *is_revealed || room.reveal_before_entry;
        commands.entity(room_entity)
            .remove::<ToSpawnRoom>()
            .insert_bundle(VisibilityBundle {
                visibility: Visibility { is_visible: is_revealed },
                ..VisibilityBundle::default()
            })
            .insert(Name::from(room_name.clone()))
            .insert_bundle(TransformBundle {
                local: Transform::from_translation(room.pos),
                ..TransformBundle::default()
            })
            .add_children(|parent| {
                for geometry in room.geometry.iter() {
                    let mut layer_offset = 0.;
                    parent.spawn()
                        .insert_bundle(VisibilityBundle::default())
                        .insert_bundle(TransformBundle {
                            local: Transform::from_translation(geometry.pos).with_rotation(
                                Quat::from_rotation_x(geometry.rotation.x) *
                                Quat::from_rotation_y(geometry.rotation.y) *
                                Quat::from_rotation_z(geometry.rotation.z)
                            ),
                            ..TransformBundle::default()
                        })
                        .insert(Name::new(geometry.label.as_ref().cloned().unwrap_or(format!("unnamed {}", geometry.shape.name()))))
                        .add_children(|parent| {
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
                                }).mk_mesh(&asset_server, &texmat, Vec3::Z * layer_offset / 2., AtlasIndex::default()));
                                parent.spawn()
                                    .insert(InRoom { room: room_name.clone() })
                                    .insert_bundle(PbrBundle {
                                        mesh,
                                        material,
                                        //transform: Transform::from_translation(geometry.offset + Vec3::Z * layer_offset / 2.),//.with_scale(Vec3::splat(layer_offset + 1.)),
                                        ..default()
                                    });
                                layer_offset += 0.001;
                            }
                        });
                }

                for light in room.lights.iter() {
                    let mut light_builder = parent.spawn();
                    light.insert_bundle(&mut light_builder, Vec3::ZERO);
                }

                for prefab in room.prefabs.iter() {
                    if prefab.room_child {
                        let path = fix_missing_extension::<PrefabLoader>(prefab.asset.clone());
                        parent.spawn()
                            .insert_bundle(TransformBundle {
                                local: Transform::from_translation(match prefab.at {
                                    PrefabLocation::Free(v) => v,
                                    _ => { unimplemented!() },
                                }).with_rotation(Quat::from_euler(EulerRot::XYZ, prefab.rotation.x, prefab.rotation.y, prefab.rotation.z)),
                                ..default()
                            })
                            .insert_bundle(VisibilityBundle::default())
                            .insert(ToInitHandle::<Prefab>::new(asset_server.load(&path)))
                            .insert(prefab.script_vars.clone())
                            .insert(Name::new(prefab.label.as_ref().cloned().unwrap_or(format!("unnamed {}", prefab.asset))))
                            .insert_if(prefab.attributes.is_some(), || prefab.attributes.as_ref().unwrap().clone());
                    } else {
                        todo!();
                    }
                }
            });

        if is_revealed {
            let mut args = HashMap::<String, ScriptVar>::new();
            args.insert("name".into(), room_name.clone().into());
            args.insert("entity".into(), room_entity.clone().into());
            commands.entity(room_entity).insert(AwaitScript {
                script_ids: waiting_scripts.clone(),
                event: LuaEvent {
                    hook_name: ON_ROOM_REVEAL.into(),
                    args: (args,).into(),
                    recipients: Recipients::All,
                },
            });
        }
    }
}