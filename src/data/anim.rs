use std::collections::HashMap;

use bevy::{prelude::*, render::{mesh::{Mesh}}, time::Timer, utils::{default, Uuid}, scene::{SceneBundle, Scene}, ecs::system::EntityCommands};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{system::common::ToInit, scripting::random::random_range};
use crate::{system::texture::{MaterialColors, Background}};

use super::{geometry::Shape, material::*};

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
pub struct Bone {
    /// Name of the bone; this does not need to be unique if no parts are attached to these bones (but unique children might still have attachments)
    pub name: String,
    /// The offset in units from the parent's summed offset
    #[serde(default)]
    pub offset: Vec3,
    /// The Euler rotation of this bone from the parent's summed rotation
    #[serde(default)]
    pub rotation: Vec3,
    /// The scale of attachments to this bone, which is multiplied by all the parent's scales
    #[serde(default)]
    pub scale: Vec3,
    /// Whether this bone, its attachments, and its children are visible by default
    #[serde(default = "default_is_visible")]
    pub is_visible: bool,
    /// The child bones
    #[serde(default)]
    pub children: Vec<Box<Bone>>,
}
fn default_is_visible() -> bool { true }
impl Default for Bone {
    fn default() -> Self {
        Bone {
            name:      "root".into(),
            offset:     Vec3::ZERO,
            rotation:   Vec3::ZERO,
            scale:      Vec3::ZERO,
            is_visible: true,
            children:   vec![],
        }
    }
}
#[derive(Clone, Component, Debug, Default, Eq, PartialEq)]
pub struct SkeletonRef {
    pub uuid:     Uuid,
    pub entities: HashMap<String, Vec<Entity>>,
}
impl SkeletonRef {
    pub fn spawn(commands: &mut EntityCommands, bone: &Bone) -> Self {
        let mut skeleton = SkeletonRef {
            uuid: Uuid::from_u128(random_range(u128::MIN, u128::MAX)),
            entities: HashMap::new(),
        };
        skeleton.add_bone(commands, bone, true);
        skeleton
    }

    pub fn add_bone(&mut self, commands: &mut EntityCommands, bone: &Bone, is_visible: bool) {
        commands.add_children(|parent| {
            let mut builder = parent.spawn();
            if let Some(v) = self.entities.get_mut(&bone.name) {
                v.push(builder.id());
            } else {
                self.entities.insert(bone.name.clone(), vec![builder.id()]);
            }

            let is_visible = is_visible && bone.is_visible;
            builder
                .insert(Name::new(format!("{}/{}", self.uuid, bone.name)))
                .insert_bundle(VisibilityBundle {
                    visibility: Visibility { is_visible },
                    ..default()
                })
                .insert_bundle(TransformBundle {
                    local: Transform::from_translation(bone.offset)
                        .with_rotation(Quat::from_euler(EulerRot::XYZ, bone.rotation.x, bone.rotation.y, bone.rotation.z)),
                    ..default()
                });
            for child in bone.children.iter() {
                self.add_bone(&mut builder, child.as_ref(), is_visible);
            }
        });
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BoneAttachment {
    /// Name of the bone this is attached to
    pub name: String,
    /// The offset in units from the part offset to where the bone is (for rotation, translation, etc.)
    #[serde(default)]
    pub offset: Vec3,
    /// The scale of the part attached to this bone
    #[serde(default = "default_scale")]
    pub scale: Vec3,
}
pub fn default_scale() -> Vec3 { Vec3::ONE }

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SpritePart {
    #[serde(default)]
    pub bone:  BoneAttachment,
    pub layer: ColorLayer,
    pub shape: Shape,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub atlas_offset: AtlasIndex,
    #[serde(default)]
    pub one_sided: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScenePart {
    #[serde(default)]
    pub bone:  BoneAttachment,
    pub asset: String,
    #[serde(default)]
    pub mat_overrides: HashMap<String, String>,
}
#[derive(Clone, Component, Debug, PartialEq, Default)]
pub struct SceneOverride {
    pub mat_overrides: HashMap<String, Handle<StandardMaterial>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CollisionPart {
    #[serde(default)]
    pub bone:  BoneAttachment,
    pub shape: Shape,
    pub lock_axis: [bool; 3],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum AnimPart {
    Sprite(SpritePart),
    Scene(ScenePart),
    Collision(CollisionPart),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Animation {
    pub parts:       IndexMap<String, AnimPart>,
    pub frames:      HashMap<String, Vec<Frame>>,
    pub materials:   HashMap<String, TextureMaterial>,
    #[serde(default)]
    pub skeleton:    Bone,
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
        entity:       Entity,
        commands:     &mut Commands,
        mat_colors:   &mut MaterialColors,
        asset_server: &AssetServer,
        tex_mat_info: &mut TexMatInfo,
        background:   &Background,
        meshes:       &mut Assets<Mesh>,
        materials:    &mut Assets<StandardMaterial>,
    ) -> LoadedMaterials {
        let mut by_name = HashMap::new();
        let mut layer = 0.;
        let skeleton = SkeletonRef::spawn(&mut commands.entity(entity), &self.skeleton);
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
        
                    let shape = match &part.shape {
                        Shape::Quad { w, h, d, one_sided } => Shape::Quad { w: *w, h: *h, d: d + layer, one_sided: *one_sided },
                        s => s.clone(),
                    };
                    let mesh = meshes.add(shape.mk_mesh(asset_server, &texmat, part.bone.offset + (Vec3::Y * part.shape.height() / 2.), part.atlas_offset));
        
                    let parent = if let Some(entities) = skeleton.entities.get(&part.bone.name) {
                        if entities.is_empty() {
                            warn!("SpritePart {} is attached to non-existant bone {}; will be placed at entity root", part_name, part.bone.name);
                            entity
                        } else {
                            if entities.len() > 1 {
                                warn!("SpritePart {} is attached to non-unique bone {}; will be placed at first found one", part_name, part.bone.name);
                            }
                            entities[0]
                        }
                    } else {
                        warn!("SpritePart {} is attached to non-existant bone {}; will be placed at entity root", part_name, part.bone.name);
                        entity
                    };
                    commands.entity(parent).add_children(|parent| {
                        parent.spawn().insert_bundle(PbrBundle {
                                mesh,
                                transform: Transform::from_translation(Vec3::Y * 0.00001).with_scale(part.bone.scale),
                                material,
                                ..default()
                        });
                    });
                    layer += 0.00001;
                },
                AnimPart::Scene(part) => {
                    let mut mat_overrides = HashMap::new();

                    for (name, mat_name) in part.mat_overrides.iter() {
                        let (material, _) = self.get_tex(mat_name, asset_server, tex_mat_info, background, materials);
                        by_name.insert(mat_name.clone(), material.clone());
                        mat_overrides.insert(name.clone(), material);
                    }

                    let parent = if let Some(entities) = skeleton.entities.get(&part.bone.name) {
                        if entities.is_empty() {
                            warn!("SpritePart {} is attached to non-existant bone {}; will be placed at entity root", part_name, part.bone.name);
                            entity
                        } else {
                            if entities.len() > 1 {
                                warn!("SpritePart {} is attached to non-unique bone {}; will be placed at first found one", part_name, part.bone.name);
                            }
                            entities[0]
                        }
                    } else {
                        warn!("SpritePart {} is attached to non-existant bone {}; will be placed at entity root", part_name, part.bone.name);
                        entity
                    };
                    commands.entity(parent).add_children(|parent| {
                        parent.spawn()
                            .insert_bundle(SceneBundle  {
                                scene: asset_server.load(&part.asset),
                                transform: Transform::from_translation(part.bone.offset).with_scale(part.bone.scale),
                                ..default()
                            }).insert(SceneOverride {
                                mat_overrides,
                            }).insert(ToInit::<Scene>::default());
                    });
                },
                AnimPart::Collision(part) => {

                },
            }
        }
        commands.entity(entity).insert(skeleton);
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
    pub flat_skeleton:   HashMap<String, Entity>,
}

impl AnimationState {
    pub fn new(anim: &Animation, animname: &String, flat_skeleton: HashMap<String, Entity>) -> AnimationState {
        let mut st = AnimationState {
            timer:           Timer::from_seconds(100., true),
            animname:        animname.clone(),
            frame_idx:       0,
            atlas_idx:       AtlasIndex::default(),
            time_boundaries: Vec::new(),
            flat_skeleton,
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