use std::{collections::HashMap};

use bevy::{prelude::*};
use bevy_mod_scripting::{prelude::*};
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::data::{level::*, material::{TextureMaterial, AtlasIndex}, geometry::Shape, prefab::ToSpawnPrefab};

use super::texture::{MissingTexture, Background, ImageDescriptions};

#[derive(Clone, Debug, Default)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .init_resource::<LoadingLevel>()
            .add_startup_system(startup)
            .add_system(load_level
                .run_if_resource_exists::<LoadingLevel>())
            .add_system(reset_loaded_level
                .run_if_resource_exists::<LoadedLevel>());
    }
}

#[derive(Default)]
pub struct LoadingLevel {
    pub handle: Handle<Level>,
}

pub fn startup(mut st: ResMut<LoadingLevel>, asset_server: Res<AssetServer>) {
    st.handle = asset_server.load("levels/testing/testing_house.level.ron");
}

pub fn load_level(
    st:            Res<LoadingLevel>,
    mut commands:  Commands,
    levels:        Res<Assets<Level>>,
) {
    if let Some(level) = levels.get(&st.handle) {

        let level_entity = commands.spawn().id();

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
    missing_tex:      Res<MissingTexture>,
    background:       Res<Background>,
    mut ccolor:       ResMut<ClearColor>,
    mut descriptions: ResMut<ImageDescriptions>,
    asset_server:     Res<AssetServer>,
    mut meshes:       ResMut<Assets<Mesh>>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
) {
    if st.should_reset {
        let level = &st.level;

        ccolor.0 = level.background;
        {
            let mut mat = materials.get_mut(&background.material).unwrap();
            mat.base_color = level.background;
        }

        let background_texmat = TextureMaterial::BACKGROUND;
        let missing_texmat    = TextureMaterial::MISSING;
        let lvl_mats: HashMap<&String, Handle<StandardMaterial>> = level.materials.iter()
            .map(|(name, mat)| (name, materials.add(mat.make_material(&asset_server, descriptions.as_mut()))))
            .collect();
        
        for (room_name, room) in level.rooms.iter() {
            commands.spawn().insert_bundle(VisibilityBundle::default())
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
                            }).add_children(|parent| {
                                for texname in geometry.materials.iter() {
                                    let (material, texmat) = if texname == "background" {
                                        (background.material.clone(), &background_texmat)
                                    } else if let Some(h) = lvl_mats.get(texname) {
                                        (h.clone(), &level.materials[texname])
                                    } else {
                                        warn!("room {} was unable to find material {}; using default", room_name, texname);
                                        (missing_tex.material.clone(), &missing_texmat)
                                    };
                                    let mesh = meshes.add((match geometry.shape.clone() {
                                        Shape::Quad { w, h, d } => Shape::Quad { w, h, d: d + layer_offset },
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
                            parent.spawn()
                                .insert_bundle(TransformBundle {
                                    local: Transform::from_translation(match prefab.at {
                                        PrefabLocation::Free(v) => v,
                                        _ => { unimplemented!() },
                                    }),
                                    ..default()
                                })
                                .insert_bundle(VisibilityBundle::default())
                                .insert(ToSpawnPrefab { handle: asset_server.load(&prefab.asset) });
                        } else {
                            todo!();
                        }
                    }
                });
        }
        
        commands.entity(st.level_entity)
            .remove::<ScriptCollection::<LuaFile>>()
            .insert(ScriptCollection::<LuaFile> {
                scripts: level.scripts.iter().map(|path| {
                    let handle = asset_server.load::<LuaFile, _>(path);
                    Script::<LuaFile>::new(path.clone(), handle)
                }).collect()
            });
        st.should_reset = false;
    }
}