use std::collections::HashMap;

use bevy::{prelude::{Vec2, warn, Color, Vec3, ChildBuilder, Handle, Image, StandardMaterial, AlphaMode, PbrBundle, Assets, Visibility, Transform, AssetServer}, render::{mesh::{Mesh, Indices}, render_resource::PrimitiveTopology}, time::Timer, utils::default};
use serde::{Deserialize, Serialize};

use crate::{util::serialize::*, system::texture::{MaterialColors, ImageDescriptions, Background}};

use super::{geometry::*, material::*};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct AxisChange {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Frame {
    pub delay:  f32,
    #[serde(default)]
    pub image:  AtlasIndex,
    #[serde(default)]
    pub offset: AxisChange,
    #[serde(default)]
    pub scale:  AxisChange,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ColorLayer {
    NoChange,
    Outline,
    Background,
}

impl Default for ColorLayer {
    fn default() -> Self { ColorLayer::Outline }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpritePart {
    pub layer: ColorLayer,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub atlas_offset: AtlasIndex,
    #[serde(default)]
    pub pos_offset: Vec2,
    #[serde(default = "default_start_enabled")]
    pub start_enabled: bool,
}
fn default_start_enabled() -> bool { true }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attachment {
    #[serde(default = "default_attachment_scale")]
    pub scale: Vec2,
    pub pos:   Vec3,
}
fn default_attachment_scale() -> Vec2 { Vec2::ONE }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Animation {
    pub atlas:  Atlas,
    pub parts:  HashMap<String, SpritePart>,
    pub frames: HashMap<String, Vec<Frame>>,
    pub height: f32,
    #[serde(default)]
    pub attachments: HashMap<String, Attachment>,
    pub materials:   HashMap<String, TextureMaterial>,
}

impl Animation {
    pub fn add_parts(
        &self,
        parent:       &mut ChildBuilder,
        mat_colors:   &mut MaterialColors,
        asset_server: &AssetServer,
        descriptions: &mut ImageDescriptions,
        background:   &Background,
        meshes:       &mut Assets<Mesh>,
        materials:    &mut Assets<StandardMaterial>,
    ) {
        let mut layer = 0.;
        for (part_name, part) in self.parts.iter() {
            let (material, tex_mat) = if let Some(mat_name) = part.material.as_ref() {
                if mat_name.as_str() == "background" {
                    (background.material.clone(), TextureMaterial::BACKGROUND)
                } else if let Some(mat) = self.materials.get(mat_name) {
                    (materials.add(mat.make_material(asset_server, descriptions)), mat.clone())
                } else {
                    warn!("Material \"{}\" not found for Animation SpritePart {}", mat_name, part_name);
                    (materials.add(StandardMaterial {
                        unlit: true,
                        ..default()
                    }), TextureMaterial::MISSING)
                }
            } else {
                if part.layer == ColorLayer::Background {
                    (background.material.clone(), TextureMaterial::BACKGROUND)
                } else {
                    warn!("No material given for Animation SpritePart {}", part_name);
                    (materials.add(StandardMaterial {
                        unlit: true,
                        ..default()
                    }), TextureMaterial::MISSING)
                }
            };
            
            mat_colors.layers.insert(material.clone_weak(), part.layer);

            let units_per_pixel = if self.atlas.height <= 0. { 0. } else { self.height / self.atlas.height };
            let pos = Vec3::new(
                if self.atlas.height <= 0. { 0. } else { part.pos_offset.x * units_per_pixel },
                if self.atlas.width  <= 0. { 0. } else { part.pos_offset.y * units_per_pixel },
                0.,
            );

            let shape = Shape::Quad {
                w: if self.atlas.height <= 0. { 0. } else { self.height * (self.atlas.width / self.atlas.height) },
                h: self.height
            };
            let mesh = meshes.add(shape.mk_mesh(&tex_mat, part.pos_offset.extend(layer) + pos + (Vec3::Y * self.height / 2.), part.atlas_offset));

            parent.spawn().insert_bundle(PbrBundle {
                    mesh,
                    transform: Transform::from_translation(Vec3::Y * 0.00001),
                    material,
                    ..default()
                }).insert(Visibility { is_visible: part.start_enabled });
            layer += 0.000001;
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnimationState {
    pub timer:           Timer,
    pub atlas:           Atlas,
    pub atlas_idx:       AtlasIndex,
    pub animname:        String,
    pub frame_idx:       usize,
    pub time_boundaries: Vec<f32>,
}

impl AnimationState {
    pub fn new(anim: &Animation, animname: &String) -> AnimationState {
        let mut st = AnimationState {
            timer:           Timer::from_seconds(100., true),
            animname:        animname.clone(),
            atlas:           anim.atlas,
            frame_idx:       0,
            atlas_idx:       AtlasIndex::default(),
            time_boundaries: Vec::new(),
        };
        st.set_anim(anim, animname);
        st
    }

    pub fn set_anim(&mut self, anim: &Animation, animname: &String) {
        if let Some(frames) = anim.frames.get(animname) {
            let mut time_boundaries = Vec::new();
            let mut cur_time = 0.;
            let mut atlas_idx = None;
            for frame in frames {
                if atlas_idx.is_none() {
                    atlas_idx = Some(frame.image);
                }
                cur_time += frame.delay;
                time_boundaries.push(cur_time);
            }
            self.timer = Timer::from_seconds(if cur_time <= 0. { 100000. } else { cur_time }, true);
            self.frame_idx = 0;
            self.atlas_idx = atlas_idx.unwrap_or(AtlasIndex::default());
        } else {
            warn!("No frames found for animation `{}`, state is unable to change", animname);
        }
    }
}