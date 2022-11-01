use std::f32::consts::FRAC_PI_4;

use bevy::{prelude::*, render::{mesh::{Mesh, Indices}, render_resource::{PrimitiveTopology}}};
use serde::{Deserialize, Serialize};

use crate::{util::serialize::*};

use super::{material::*};

#[derive(Clone, Debug)]
pub struct AnimatedMesh {
    pub mesh: Mesh,
    pub uv_frames: Vec<Vec<[f32; 2]>>,
}

#[derive(Clone, Default)]
pub struct MeshBuilder {
    pub vertices: Vec<[f32; 3]>,
    pub normals:  Vec<[f32; 3]>,
    pub uvs:      Vec<[f32; 2]>,
    pub indices:  Vec<u32>,
}

impl MeshBuilder {
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn push(&mut self, vertex: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> &mut Self {
        self.vertices.push(vertex);
        self.normals.push(normal);
        self.uvs.push(uv);
        self
    }

    pub fn push_indices(&mut self, mut indices: Vec<u32>) -> &mut Self {
        self.indices.append(&mut indices);
        self
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(self.indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.generate_tangents().unwrap();
        mesh
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Shape {
    Box { w: f32, h: f32, d: f32 },
    Quad { w: f32, h: f32 },
}

impl Shape {
    pub fn mk_mesh(&self, mat: &TextureMaterial, offset: Vec3, idx: AtlasIndex) -> Mesh {
        let mut builder = MeshBuilder::default();
        let [uv_left, uv_right, uv_top, uv_bottom] = mat.get_uvs(idx);
        match self {
            Shape::Box {w, h, d} => {
                // let max_x = w / 2.;
                // let max_y = h / 2.;
                // let max_z = d / 2.;
                // let min_x = -max_x;
                // let min_y = -max_y;
                // let min_z = -max_z;

                // for i in 0..(w.ceil().abs() as i32) {
                //     ([min_x, min_y, max_z], [0., 0., 1.0], [0., 0.]),
                //     ([max_x, min_y, max_z], [0., 0., 1.0], [1.0, 0.]),
                //     ([max_x, max_y, max_z], [0., 0., 1.0], [1.0, 1.0]),
                //     ([min_x, max_y, max_z], [0., 0., 1.0], [0., 1.0]),
                // }

                // let vertices = &[
                //     // Top
                //     ([min_x, min_y, max_z], [0., 0., 1.0], [0., 0.]),
                //     ([max_x, min_y, max_z], [0., 0., 1.0], [1.0, 0.]),
                //     ([max_x, max_y, max_z], [0., 0., 1.0], [1.0, 1.0]),
                //     ([min_x, max_y, max_z], [0., 0., 1.0], [0., 1.0]),
                //     // Bottom
                //     ([min_x, max_y, min_z], [0., 0., -1.0], [1.0, 0.]),
                //     ([max_x, max_y, min_z], [0., 0., -1.0], [0., 0.]),
                //     ([max_x, min_y, min_z], [0., 0., -1.0], [0., 1.0]),
                //     ([min_x, min_y, min_z], [0., 0., -1.0], [1.0, 1.0]),
                //     // Right
                //     ([max_x, min_y, min_z], [1.0, 0., 0.], [0., 0.]),
                //     ([max_x, max_y, min_z], [1.0, 0., 0.], [1.0, 0.]),
                //     ([max_x, max_y, max_z], [1.0, 0., 0.], [1.0, 1.0]),
                //     ([max_x, min_y, max_z], [1.0, 0., 0.], [0., 1.0]),
                //     // Left
                //     ([min_x, min_y, max_z], [-1.0, 0., 0.], [1.0, 0.]),
                //     ([min_x, max_y, max_z], [-1.0, 0., 0.], [0., 0.]),
                //     ([min_x, max_y, min_z], [-1.0, 0., 0.], [0., 1.0]),
                //     ([min_x, min_y, min_z], [-1.0, 0., 0.], [1.0, 1.0]),
                //     // Front
                //     ([max_x, max_y, min_z], [0., 1.0, 0.], [1.0, 0.]),
                //     ([min_x, max_y, min_z], [0., 1.0, 0.], [0., 0.]),
                //     ([min_x, max_y, max_z], [0., 1.0, 0.], [0., 1.0]),
                //     ([max_x, max_y, max_z], [0., 1.0, 0.], [1.0, 1.0]),
                //     // Back
                //     ([max_x, min_y, max_z], [0., -1.0, 0.], [0., 0.]),
                //     ([min_x, min_y, max_z], [0., -1.0, 0.], [1.0, 0.]),
                //     ([min_x, min_y, min_z], [0., -1.0, 0.], [1.0, 1.0]),
                //     ([max_x, min_y, min_z], [0., -1.0, 0.], [0., 1.0]),
                // ];
        
                // let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
                // let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
                // let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();
        
                // let indices = Indices::U32(vec![
                //     0, 1, 2, 2, 3, 0, // top
                //     4, 5, 6, 6, 7, 4, // bottom
                //     8, 9, 10, 10, 11, 8, // right
                //     12, 13, 14, 14, 15, 12, // left
                //     16, 17, 18, 18, 19, 16, // front
                //     20, 21, 22, 22, 23, 20, // back
                // ]);
        
                // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                // mesh.set_indices(Some(indices));
            },
            Shape::Quad {w, h} => {
                let extent_x = w * 0.5;
                let extent_y = h * 0.5;
                match mat.mode {
                    MaterialMode::Stretch => {
                        builder.push_indices(vec![0, 2, 1, 0, 3, 2]);
                        let min_x = -extent_x + offset.x;
                        let max_x =  extent_x + offset.x;
                        let min_y = -extent_y + offset.y;
                        let max_y =  extent_y + offset.y;
                        builder.push([min_x, min_y, offset.z], [0., 0., 1.], [uv_left,  uv_bottom]);
                        builder.push([min_x, max_y, offset.z], [0., 0., 1.], [uv_left,  uv_top]);
                        builder.push([max_x, max_y, offset.z], [0., 0., 1.], [uv_right, uv_top]);
                        builder.push([max_x, min_y, offset.z], [0., 0., 1.], [uv_right, uv_bottom]);
                    },
                    MaterialMode::Repeat { step, on_step } => {
                        for y in 0..((h / step).abs().ceil() as i32) {
                            let offset_y = step * y as f32 + offset.y;
                            let min_y = -extent_y + offset_y;
                            let max_y = (-extent_y + offset_y + step).min(extent_y + offset.y);
                            for x in 0..((w / step).abs().ceil() as i32) {
                                let offset_x = step * x as f32 + offset.x;
                                let i = builder.len() as u32;
                                builder.push_indices(vec![i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                                let min_x = -extent_x + offset_x;
                                let max_x = (-extent_x + offset_x + step).min(extent_x + offset.x);
                                let [uv_top, uv_bottom, uv_left, uv_right] = match on_step {
                                    RepeatType::Rotate180 if (x + y) % 2 == 0 => [uv_right, uv_left, uv_bottom, uv_top],
                                    _ => [uv_left, uv_right, uv_top, uv_bottom],
                                };
                                builder.push([min_x, min_y, offset.z], [0., 0., 1.], [uv_left,  uv_bottom]);
                                builder.push([min_x, max_y, offset.z], [0., 0., 1.], [uv_left,  uv_top]);
                                builder.push([max_x, max_y, offset.z], [0., 0., 1.], [uv_right, uv_top]);
                                builder.push([max_x, min_y, offset.z], [0., 0., 1.], [uv_right, uv_bottom]);
                            }
                        }
                    },
                }
            },
        }
        builder.build()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Geometry {
    #[serde(default)]
    pub label:     Option<String>,
    pub pos:       Vec3,
    #[serde(default)]
    pub offset:    Vec3,
    #[serde(default)]
    pub rotation:  Vec3,
    pub shape:     Shape,
    pub materials: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum LightKind {
    Directional {
        #[serde(default = "default_illuminance")]
        illuminance: f32,
        #[serde(default = "default_length")]
        length: f32,
    },
    Point {
        #[serde(default = "default_intensity")]
        intensity: f32,
        #[serde(default = "default_range")]
        range:     f32,
        #[serde(default = "default_radius")]
        radius:    f32,
    },
    SpotLight {
        target:    Vec3,
        #[serde(default = "default_intensity")]
        intensity: f32,
        #[serde(default = "default_range")]
        range:     f32,
        #[serde(default = "default_radius")]
        radius:    f32,
        #[serde(default = "default_inner_angle")]
        inner_angle: f32,
        #[serde(default = "default_outer_angle")]
        outer_angle: f32,
    },
}
pub fn default_illuminance() -> f32 { 100000. }
pub fn default_length() -> f32 { 256. }
pub fn default_intensity() -> f32 { 800. }
pub fn default_range() -> f32 { 20. }
pub fn default_radius() -> f32 { 0. }
pub fn default_inner_angle() -> f32 { 0. }
pub fn default_outer_angle() -> f32 { FRAC_PI_4 }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Light {
    pub label: Option<String>,
    pub pos:   Vec3,
    pub kind:  LightKind,
    #[serde(default = "default_color", deserialize_with = "deserialize_hex_color", serialize_with = "serialize_hex_color")]
    pub color: Color,
    #[serde(default)]
    pub shadows_enabled: bool,
    #[serde(default = "default_shadow_depth_bias")]
    pub shadow_depth_bias: f32,
    #[serde(default = "default_shadow_normal_bias")]
    pub shadow_normal_bias: f32,
}
fn default_color() -> Color { Color::WHITE }
pub fn default_shadow_depth_bias() -> f32 { 0.02 }
pub fn default_shadow_normal_bias() -> f32 { 0.6 }
impl Light {
    pub fn spawn(&self, commands: &mut Commands, offset: Vec3) -> Entity {
        match self.kind {
            LightKind::Point { intensity, range, radius } => {
                commands.spawn().insert_bundle(PointLightBundle {
                    point_light: PointLight {
                        intensity,
                        range,
                        radius,
                        color: self.color,
                        shadows_enabled: self.shadows_enabled,
                        shadow_depth_bias: self.shadow_depth_bias,
                        shadow_normal_bias: self.shadow_normal_bias,
                    },
                    transform: Transform::from_translation(offset + self.pos),
                    ..default()
                }).id()
            },
            LightKind::SpotLight { target, intensity, range, radius, inner_angle, outer_angle } => {
                commands.spawn().insert_bundle(SpotLightBundle {
                    spot_light: SpotLight {
                        intensity,
                        range,
                        radius,
                        outer_angle,
                        inner_angle,
                        color: self.color,
                        shadows_enabled: self.shadows_enabled,
                        shadow_depth_bias: self.shadow_depth_bias,
                        shadow_normal_bias: self.shadow_normal_bias,
                    },
                    transform: Transform::from_translation(offset + self.pos).looking_at(target, Vec3::Y),
                    ..default()
                }).id()
            },
            LightKind::Directional { illuminance, length } => {
                commands.spawn().insert_bundle(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        illuminance,
                        color: self.color,
                        shadows_enabled: self.shadows_enabled,
                        shadow_depth_bias: self.shadow_depth_bias,
                        shadow_normal_bias: self.shadow_normal_bias,
                        shadow_projection: OrthographicProjection {
                            left: -length,
                            right: length,
                            bottom: -length,
                            top: length,
                            near: -length,
                            far: length,
                            ..Default::default()
                        },
                    },
                    transform: Transform::from_translation(offset + self.pos),
                    ..default()
                }).id()
            },
        }
    }
}