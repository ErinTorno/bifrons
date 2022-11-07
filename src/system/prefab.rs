use bevy::{prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_scripting::prelude::{ScriptCollection, LuaFile, Script};
use crate::{system::common::ToInit, data::{prefab::*, input::{ActionState, InputMap}, material::TexMatInfo}, scripting::LuaScriptVars};

use super::{texture::{MaterialColors, Background}, camera::{ActiveCamera, Focus}, common::ToInitHandle};

#[derive(Clone, Debug, Default)]
pub struct PrefabPlugin;

impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_inspectable::<Player>()
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
    commands.spawn()
        .insert_bundle(TransformBundle {
            ..default()
        })
        .insert_bundle(VisibilityBundle::default())
        .insert(Player { id: 0 })
        .insert(LuaScriptVars::default())
        .insert(ToInitHandle::<Prefab>::new(asset_server.load("chars/labolas.prefab.ron")));
}

pub fn spawn_prefab(
    mut commands:     Commands,
    mut mat_colors:   ResMut<MaterialColors>,
    asset_server:     Res<AssetServer>,
    background:       Res<Background>,
    mut tex_mat_info: ResMut<TexMatInfo>,
    prefabs:          Res<Assets<Prefab>>,
    mut meshes:       ResMut<Assets<Mesh>>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
    mut to_spawn:     Query<(Entity, &ToInitHandle<Prefab>, &mut LuaScriptVars, Option<&Player>)>,
) {
    for (entity, ToInitHandle(handle), mut script_vars, player) in to_spawn.iter_mut() {
        if let Some(prefab) = prefabs.get(handle) {
            let entity = commands.entity(entity)
                .insert(ActionState::default())
                .insert(InputMap::default())
                .remove::<ToInitHandle<Prefab>>()
                .id();
            let loaded = prefab.animation.add_parts(&mut commands.entity(entity), mat_colors.as_mut(), &asset_server, &mut tex_mat_info, &background, meshes.as_mut(), materials.as_mut());
            commands.entity(entity).insert(loaded);

            if !prefab.scripts.is_empty() {
                script_vars.merge(&prefab.script_vars);
                commands.entity(entity)
                    .insert(ScriptCollection::<LuaFile> {
                        scripts: prefab.scripts.iter().map(|path| {
                            let handle = asset_server.load::<LuaFile, _>(path);
                            Script::<LuaFile>::new(path.clone(), handle)
                        }).collect()
                    });
            }
            if player.is_some() {
                commands.spawn()
                    .insert(ToInit::<Camera3d>::default())
                    .insert(ActiveCamera {
                        controller: Some(entity),
                        focus: Focus::Entity { which: entity, offset: Vec3::ZERO }
                    });
            }
        }
    }
}