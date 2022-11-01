use std::{collections::HashMap, f32::consts::PI};

use bevy::{prelude::*, render::{render_resource::{FilterMode, SamplerDescriptor}, texture::ImageSampler}};
use bevy_mod_scripting::{prelude::*};
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::data::{level::*, geometry::{TextureMaterial, AtlasIndex}};

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
            for geometry in room.geometry.iter() {
                for texname in geometry.materials.iter() {
                    let (material, texmat) = if texname == "background" {
                        (background.material.clone(), &background_texmat)
                    } else if let Some(h) = lvl_mats.get(texname) {
                        (h.clone(), &level.materials[texname])
                    } else {
                        warn!("room {} was unable to find material {}; using default", room_name, texname);
                        (missing_tex.material.clone(), &missing_texmat)
                    };
                    commands.spawn()
                        .insert(InRoom { room: room_name.clone() })
                        .insert_bundle(PbrBundle {
                            mesh: meshes.add(geometry.shape.mk_mesh(texmat, geometry.offset, AtlasIndex::default())),
                            material,
                            transform: Transform::from_translation(room.pos + geometry.pos).with_rotation(
                                Quat::from_rotation_x(geometry.rotation.x) *
                                Quat::from_rotation_y(geometry.rotation.y) *
                                Quat::from_rotation_z(geometry.rotation.z)
                            ),
                            ..default()
                        });
                }
            }

            for light in room.lights.iter() {
                let entity = light.spawn(&mut commands, room.pos);
                commands.entity(entity).insert(InRoom { room: room_name.clone() });
            }
        }
            
        // commands.spawn().insert_bundle(PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        //     material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        //     ..default()
        // });
            
        // commands.spawn().insert_bundle(PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::Plane { size: 3.0 })),
        //     transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
        //     material: materials.add(Color::rgb(0.5, 0.2, 0.5).into()),
        //     ..default()
        // });

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