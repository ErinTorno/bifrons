use bevy::{prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use crate::{system::common::ToInit, data::{prefab::*, input::{ActionState, InputMap}}};

use super::{texture::{MaterialColors, ImageDescriptions, Background}, camera::{ActiveCamera, Focus}};

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
        .insert(ToSpawnPrefab { handle: asset_server.load("chars/labolas.prefab.ron") });
}

pub fn spawn_prefab(
    mut commands:     Commands,
    mut mat_colors:   ResMut<MaterialColors>,
    asset_server:     Res<AssetServer>,
    background:       Res<Background>,
    mut descriptions: ResMut<ImageDescriptions>,
    actors:           Res<Assets<Prefab>>,
    mut meshes:       ResMut<Assets<Mesh>>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
    to_spawn:         Query<(Entity, &ToSpawnPrefab, Option<&Player>)>,
) {
    for (entity, ToSpawnPrefab { handle }, player) in to_spawn.iter() {
        if let Some(actor) = actors.get(handle) {
            commands.entity(entity)
                .insert(ActionState::default())
                .insert(InputMap::default())
                .remove::<ToSpawnPrefab>()
                .add_children(|parent| {
                    actor.animation.add_parts(parent, mat_colors.as_mut(), &asset_server, &mut descriptions, &background, meshes.as_mut(), materials.as_mut());
                });
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