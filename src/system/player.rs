use bevy::{prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use crate::{system::common::ToInit, data::{actor::*, input::{ActionState, InputMap}}};

use super::texture::{MaterialColors, ImageDescriptions, Background};

#[derive(Clone, Debug, Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_inspectable::<Player>()
            .add_startup_system(temp_setup)
            .add_system(spawn_player)
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
        .insert(Player { id: 0 })
        .insert(ToSpawnActor { handle: asset_server.load("actors/player/labolas.actor.ron") });
}

pub fn spawn_player(
    mut commands:     Commands,
    mut mat_colors:   ResMut<MaterialColors>,
    asset_server:     Res<AssetServer>,
    background:       Res<Background>,
    mut descriptions: ResMut<ImageDescriptions>,
    actors:           Res<Assets<Actor>>,
    mut meshes:       ResMut<Assets<Mesh>>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
    to_spawn:         Query<(Entity, &ToSpawnActor), With<Player>>,
) {
    for (entity, ToSpawnActor { handle }) in to_spawn.iter() {
        if let Some(actor) = actors.get(handle) {

            commands.entity(entity)
                .insert_bundle(TransformBundle {
                    ..default()
                })
                .insert_bundle(VisibilityBundle::default())
                .insert(ActionState::default())
                .insert(InputMap::default())
                .remove::<ToInit<Player>>()
                .remove::<ToSpawnActor>()
                .add_children(|parent| {
                    actor.animation.add_parts(parent, mat_colors.as_mut(), &asset_server, &mut descriptions, &background, meshes.as_mut(), materials.as_mut());
                });
        }
    }
}