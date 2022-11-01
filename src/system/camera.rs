use bevy::prelude::*;

use std::{collections::HashMap, default};

use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use serde::*;

#[derive(Clone, Debug, Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_type::<Focus>()
            .register_type::<Follow>()
            .add_startup_system(setup)
            // .add_system(follow_system)
        ;
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Reflect, Serialize)]
pub enum Focus {
    Entity {
        which: Entity,
        offset: Vec3,
    },
    Free
}
impl Default for Focus {
    fn default() -> Self { Focus::Free }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Reflect, Serialize)]
pub struct Camera {
    pub focus: Focus,
}
impl Default for Camera {
    fn default() -> Self {
        Camera {
            focus: Focus::default(),
        }
    }
}

pub fn setup(
    mut commands:  Commands,
) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(4.0, 4.0, 6.0).looking_at(Vec3::new(0., 2., 0.), Vec3::Y),
        ..default()
    });
}

#[derive(Clone, Component, Debug, Deserialize, PartialEq, Reflect, Serialize)]
pub struct Follow {
    pub target: Entity,
    #[serde(default)]
    pub lock_x: bool,
    #[serde(default)]
    pub lock_y: bool,
    #[serde(default)]
    pub lock_z: bool,
}
impl Follow {
    pub fn targetting(target: Entity) -> Self {
        Follow {
            target,
            lock_x: false,
            lock_y: false,
            lock_z: false,
        }
    }
}

// pub fn follow_system(
//     world: &World,
//     mut query: Query<(&mut Transform, &Follow)>,
// ) {
//     for (mut transform, follow) in query.iter_mut() {
//         if let Some(target) = world.get::<Transform>(follow.target) {

//             transform.look_at(target.translation, Vec3::Y);
//         }
//     }
// }