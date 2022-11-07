use std::collections::HashMap;

use bevy::{prelude::{Vec2, warn, Color, Vec3, ChildBuilder, Handle, Image, StandardMaterial, AlphaMode, PbrBundle, Assets, Visibility, Transform, AssetServer, Component, BuildChildren}, render::{mesh::{Mesh, Indices}, render_resource::PrimitiveTopology}, time::Timer, utils::default, scene::{SceneBundle, Scene}, ecs::system::EntityCommands};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::system::common::ToInit;
use crate::{util::serialize::*, system::texture::{MaterialColors, Background}};

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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpritePart {
    pub layer: ColorLayer,
    pub shape: Shape,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScenePart {
    pub asset: String,
    #[serde(default = "default_scene_scale")]
    pub scale: Vec3,
    #[serde(default)]
    pub offset: Vec3,
    #[serde(default)]
    pub mat_overrides: HashMap<String, String>,
}
pub fn default_scene_scale() -> Vec3 { Vec3::ONE }
#[derive(Clone, Component, Debug, PartialEq, Default)]
pub struct SceneOverride {
    pub mat_overrides: HashMap<String, Handle<StandardMaterial>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum AnimPart {
    Sprite(SpritePart),
    Scene(ScenePart),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Attachment {
    #[serde(default = "default_attachment_scale")]
    pub scale: Vec2,
    pub pos:   Vec3,
}
fn default_attachment_scale() -> Vec2 { Vec2::ONE }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Animation {
    pub parts:  IndexMap<String, AnimPart>,
    pub frames: HashMap<String, Vec<Frame>>,
    #[serde(default)]
    pub attachments: HashMap<String, Attachment>,
    pub materials:   HashMap<String, TextureMaterial>,
}

impl Animation {
    fn get_tex(
        &self,
        mat_name:     &String,
        asset_server: &AssetServer,
        tex_mat_info: &mut TexMatInfo,
        background:   &Background,
        materials:    &mut Assets<StandardMaterial>,
    ) -> (Handle<StandardMaterial>, TextureMaterial) {
        if mat_name.as_str() == "background" {
            (background.material.clone(), TextureMaterial::BACKGROUND)
        } else if let Some(mat) = self.materials.get(mat_name) {
            (mat.load_material(asset_server, tex_mat_info, materials), mat.clone())
        } else {
            warn!("Material \"{}\" not found", mat_name);
            (materials.add(StandardMaterial {
                unlit: true,
                ..default()
            }), TextureMaterial::MISSING)
        }
    }

    pub fn add_parts(
        &self,
        commands:     &mut EntityCommands,
        mat_colors:   &mut MaterialColors,
        asset_server: &AssetServer,
        tex_mat_info: &mut TexMatInfo,
        background:   &Background,
        meshes:       &mut Assets<Mesh>,
        materials:    &mut Assets<StandardMaterial>,
    ) -> LoadedMaterials {
        let mut by_name = HashMap::new();
        let mut layer = 0.;
        commands.add_children(|parent| {
            for (part_name, part) in self.parts.iter() {
                match part {
                    AnimPart::Sprite(part) => {
                        let (material, texmat) = if let Some(mat_name) = part.material.as_ref() {
                            let pair = self.get_tex(mat_name, asset_server, tex_mat_info, background, materials);
                            by_name.insert(mat_name.clone(), pair.0.clone());
                            pair
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
            
                        let pos = part.pos_offset.extend(0.);
                        let shape = match &part.shape {
                            Shape::Quad { w, h, d } => Shape::Quad { w: *w, h: *h, d: d + layer },
                            s => s.clone(),
                        };
                        let mesh = meshes.add(shape.mk_mesh(asset_server, &texmat, part.pos_offset.extend(layer) + pos + (Vec3::Y * part.shape.height() / 2.), part.atlas_offset));
            
                        parent.spawn().insert_bundle(PbrBundle {
                                mesh,
                                transform: Transform::from_translation(Vec3::Y * 0.00001),
                                material,
                                ..default()
                            }).insert(Visibility { is_visible: part.start_enabled });
                        layer += 0.00001;
                    },
                    AnimPart::Scene(part) => {
                        let mut mat_overrides = HashMap::new();
    
                        for (name, mat_name) in part.mat_overrides.iter() {
                            let (material, _) = self.get_tex(mat_name, asset_server, tex_mat_info, background, materials);
                            by_name.insert(name.clone(), material.clone());
                            mat_overrides.insert(name.clone(), material);
                        }
    
                        parent.spawn()
                            .insert_bundle(SceneBundle  {
                                scene: asset_server.load(&part.asset),
                                transform: Transform::from_translation(part.offset).with_scale(part.scale),
                                ..default()
                            }).insert(SceneOverride {
                                mat_overrides,
                            }).insert(ToInit::<Scene>::default());
                    },
                }
            }
        });
        LoadedMaterials { by_name }
    }
}

#[derive(Clone, Debug)]
pub struct AnimationState {
    pub timer:           Timer,
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