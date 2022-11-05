use bevy::prelude::*;

use std::{collections::HashMap, default};

use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use serde::*;

use crate::data::input::{ActionState, InputMap};

use super::common::ToInit;

#[derive(Clone, Debug, Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_type::<Focus>()
            .register_type::<Follow>()
            .add_system(setup_camera)
            .add_system(cam_movement)
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

#[derive(Clone, Component, Debug, Deserialize, PartialEq, Reflect, Serialize)]
pub struct ActiveCamera {
    pub controller: Option<Entity>,
    pub focus: Focus,
}
impl Default for ActiveCamera {
    fn default() -> Self {
        ActiveCamera {
            controller: None,
            focus: Focus::default(),
        }
    }
}

pub fn setup_camera(
    mut commands:  Commands,
    cam_query:   Query<(Entity, &ActiveCamera, Option<&Transform>), With<ToInit<Camera3d>>>,
    actor_query: Query<&Transform, Without<ToInit<Camera3d>>>,
) {
    for (entity, cam, transform) in cam_query.iter() {
        let transform = cam.controller.and_then(|e| actor_query.get(e).ok())
            .or(transform)
            .map(|t| {
                t.clone().with_translation(Vec3::new(5., 5., 5.)).looking_at(t.translation, Vec3::Y)
            });
        
        commands.entity(entity)
            .remove::<ToInit<Camera3d>>()
            .insert_bundle(Camera3dBundle {
                transform: transform.unwrap_or(Transform::from_xyz(4.0, 4.0, 6.0).looking_at(Vec3::new(0., 2., 0.), Vec3::Y)),
                ..default()
            });
    }
}

pub fn cam_movement(
    time: Res<Time>,
    mut cam_query: Query<(&mut Transform, &ActiveCamera), With<Camera3d>>,
    actor_query: Query<(&Transform, &ActionState, &InputMap), Without<Camera3d>>,
) {
    for (mut transform, camera) in cam_query.iter_mut() {
        if let Some(entity) = camera.controller {
            if let Some((a_trans, a_state, a_inputmap)) = actor_query.get(entity).ok() {
                let mut x_speed = 0.;
                let mut y_speed = 0.;
                let mut rz_speed = 0.;
                let mut az_speed = 0.;
                rz_speed -= time.delta_seconds() * a_inputmap.zoom_speed * a_state.input_ts("zoom_in").power();
                rz_speed -= time.delta_seconds() * a_inputmap.zoom_speed * a_state.input_ts("zoom_out").power();
                az_speed -= time.delta_seconds() * a_inputmap.zoom_speed * a_state.input_ts("forward").power();
                az_speed += time.delta_seconds() * a_inputmap.zoom_speed * a_state.input_ts("back").power();
                x_speed -= time.delta_seconds() * a_inputmap.cam_speed  * a_state.input_ts("left").power();
                x_speed += time.delta_seconds() * a_inputmap.cam_speed  * a_state.input_ts("right").power();
                y_speed += time.delta_seconds() * a_inputmap.cam_speed  * a_state.input_ts("rise").power();
                y_speed -= time.delta_seconds() * a_inputmap.cam_speed  * a_state.input_ts("fall").power();
                let translation = transform.rotation * Vec3::new(x_speed, 0., rz_speed);
                let translation = translation + Quat::from_rotation_y(transform.rotation.y) * (Vec3::Z * az_speed);
                transform.translation += translation + Vec3::Y * y_speed;
                // let rotation = transform.rotation.clone();
                
                // if a_state.input_ts(s)
            }
        }
    }
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