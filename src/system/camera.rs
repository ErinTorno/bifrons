use bevy::prelude::*;
use serde::*;

use crate::data::input::{ActionState, InputMap, InputTS, InputData};

use super::common::ToInit;

#[derive(Clone, Debug, Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_type::<Focus>()
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
                t.clone().with_translation(Vec3::new(5., 5., 5.)).looking_at(Vec3::new(5., 5., 4.), Vec3::Y)
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
    windows: Res<Windows>,
    mut cam_query: Query<(&mut Transform, &ActiveCamera), With<Camera3d>>,
    mut actor_query: Query<(&Transform, &mut ActionState, &InputMap), Without<Camera3d>>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);
    for (mut transform, camera) in cam_query.iter_mut() {
        if let Some(entity) = camera.controller {
            if let Some((a_trans, mut a_state, a_inputmap)) = actor_query.get_mut(entity).ok() {
                let mut x_speed = 0.;
                let mut y_speed = 0.;
                let mut rz_speed = 0.;
                let mut az_speed = 0.;
                let m = time.delta_seconds() * if a_state.input_ts("run").is_blank() { 1. } else { 1.5 };
                rz_speed -= time.delta_seconds() * a_inputmap.zoom_speed * a_state.input_ts("zoom_in").time_scaled_power();
                rz_speed += time.delta_seconds() * a_inputmap.zoom_speed * a_state.input_ts("zoom_out").time_scaled_power();
                az_speed += m * a_inputmap.cam_speed * a_state.input_ts("forward").time_scaled_power();
                az_speed -= m * a_inputmap.cam_speed * a_state.input_ts("back").time_scaled_power();
                x_speed  -= m * a_inputmap.cam_speed * a_state.input_ts("left").time_scaled_power();
                x_speed  += m * a_inputmap.cam_speed * a_state.input_ts("right").time_scaled_power();
                y_speed  -= m * a_inputmap.cam_speed * a_state.input_ts("fall").time_scaled_power();
                y_speed  += m * a_inputmap.cam_speed * a_state.input_ts("rise").time_scaled_power();

                if let Some(InputTS { data: InputData::VecChain { vecs },.. }) = a_state.inputs.remove("cam_drag") {
                    let v = vecs.iter().map(|ts| ts.value).fold(Vec2::ZERO, |a, b| a + b);
                    let delta_x = v.x / window_size.x * std::f32::consts::PI * 2.0;
                    let delta_y = v.y / window_size.y * std::f32::consts::PI;
                    let yaw = Quat::from_rotation_y(-delta_x);
                    let pitch = Quat::from_rotation_x(-delta_y);
                    transform.rotation = yaw * transform.rotation;
                    transform.rotation = transform.rotation * pitch;
                    transform.rotation = yaw * transform.rotation;
                    transform.rotation = transform.rotation * pitch;
                }
                
                let translation = transform.rotation * Vec3::new(x_speed, 0., rz_speed);
                let target = transform.rotation * Vec3::Z;
                let target = transform.translation + Vec3::new(target.x, 0., target.z);
                let translation = translation + transform.clone().looking_at(target, Vec3::Y).rotation * (Vec3::Z * az_speed);
                // let translation = translation + Quat::from_rotation_y(transform.rotation.y) * (Vec3::Z * az_speed);
                transform.translation += translation + Vec3::Y * y_speed;
            }
        }
    }
}