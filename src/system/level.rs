use std::{collections::{HashMap, HashSet}};

use bevy::{prelude::*};
use bevy_mod_scripting::{prelude::*};
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::{data::{level::*, material::{TextureMaterial, AtlasIndex, TexMatInfo}, geometry::Shape, prefab::{PrefabLoader, Prefab}}, scripting::{random, AwaitScript, event::ON_ROOM_REVEAL, ScriptVar}};

use super::{texture::{MissingTexture, Background}, common::{fix_missing_extension, ToInitHandle}};

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
    mut tex_mat_info: ResMut<TexMatInfo>,
    asset_server:     Res<AssetServer>,
    mut meshes:       ResMut<Assets<Mesh>>,
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

        let scripts: Vec<Script<LuaFile>> = level.scripts.iter().map(|path| {
            let handle = asset_server.load::<LuaFile, _>(path);
            Script::<LuaFile>::new(path.clone(), handle)
        }).collect();

        let script_ids: HashSet<u32> = scripts.iter().map(Script::id).collect();
        
        commands.entity(st.level_entity)
            .remove::<ScriptCollection::<LuaFile>>()
            .insert(Name::from(level.name.clone()))
            .insert(ScriptCollection::<LuaFile> { scripts });

        let background_texmat = TextureMaterial::BACKGROUND;
        let missing_texmat    = TextureMaterial::MISSING;
        let lvl_mats: HashMap<&String, Handle<StandardMaterial>> = level.materials.iter()
            .map(|(name, mat)| (name, mat.load_material(&asset_server, tex_mat_info.as_mut(), materials.as_mut())))
            .collect();
        
        for (room_name, room) in level.rooms.iter() {
            let room_entity = commands.spawn().insert_bundle(VisibilityBundle {
                    visibility: Visibility { is_visible: room.reveal_before_entry },
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
                            .insert(Name::new(geometry.label.as_ref().cloned().unwrap_or("unnamed geometry".to_string())))
                            .add_children(|parent| {
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
                                .insert(Name::new(prefab.label.as_ref().cloned().unwrap_or("unnamed prefab".to_string())));
                        } else {
                            todo!();
                        }
                    }
                    parent.parent_entity()
                });

                if room.reveal_before_entry {
                    let mut args = HashMap::<String, ScriptVar>::new();
                    args.insert("name".into(), room_name.clone().into());
                    args.insert("entity".into(), room_entity.clone().into());
                    commands.entity(room_entity).insert(AwaitScript {
                        script_ids: script_ids.clone(),
                        event: LuaEvent {
                            hook_name: ON_ROOM_REVEAL.into(),
                            args: (args,).into(),
                            recipients: Recipients::All,
                        },
                    });
                }
        }
        st.should_reset = false;
    }
}