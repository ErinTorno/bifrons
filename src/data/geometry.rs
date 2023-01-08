use std::{f32::consts::FRAC_PI_4};

use bevy::{prelude::*, render::{mesh::{Mesh, Indices}, render_resource::{PrimitiveTopology}}, ecs::{system::{EntityCommands}, world::EntityMut}};
use bevy_inspector_egui::prelude::*;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{scripting::{LuaMod, bevy_api::{math::LuaVec3, LuaEntity}}};

use super::{material::*, lua::{LuaWorld, Any3}, palette::{DynColor, SingleColored}, rgba::RgbaColor};

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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Shape {
    Box {
        w: f32,
        h: f32,
        d: f32
    },
    Quad {
        w: f32,
        h: f32,
        #[serde(default = "default_quad_depth")]
        d: f32,
        #[serde(default)]
        one_sided: bool,
    },
}
pub fn default_quad_depth() -> f32 { 0.000001 }

impl Shape {
    pub fn name(&self) -> &'static str {
        static BOX: &str = "box";
        static QUAD: &str = "quad";
        match self {
            Shape::Box {..} => BOX,
            Shape::Quad {..} => QUAD,
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            Shape::Box { h, .. } => *h,
            Shape::Quad { h, .. } => *h,
        }
    }

    pub fn mk_mesh(&self, mat: &TextureMaterial, offset: Vec3, atlas_idx: AtlasIndex) -> Mesh {
        let mut builder = MeshBuilder::default();
        match self {
            Shape::Box {..} => {
                unimplemented!();
            },
            Shape::Quad { w, h, d, one_sided } => {
                let extent_x = w * 0.5;
                let extent_y = h * 0.5;
                match mat.mode {
                    MaterialMode::Stretch => {
                        let [uv_left, uv_right, uv_top, uv_bottom] = mat.get_uvs(atlas_idx);
                        let min_x   = -extent_x + offset.x;
                        let max_x   =  extent_x + offset.x;
                        let min_y   = -extent_y + offset.y;
                        let max_y   =  extent_y + offset.y;
                        let front_z = offset.z + d / 2.;
                        let back_z  = offset.z - d / 2.;
                        builder.push([min_x, min_y, front_z], [0., 0., 1.], [uv_left,  uv_bottom]);
                        builder.push([min_x, max_y, front_z], [0., 0., 1.], [uv_left,  uv_top]);
                        builder.push([max_x, max_y, front_z], [0., 0., 1.], [uv_right, uv_top]);
                        builder.push([max_x, min_y, front_z], [0., 0., 1.], [uv_right, uv_bottom]);
                        builder.push_indices(vec![0, 2, 1, 0, 3, 2]);
                        if !one_sided {
                            builder.push([min_x, max_y, back_z ], [0., 0., -1.], [uv_left,  uv_top]);
                            builder.push([min_x, min_y, back_z ], [0., 0., -1.], [uv_left,  uv_bottom]);
                            builder.push([max_x, min_y, back_z ], [0., 0., -1.], [uv_right, uv_bottom]);
                            builder.push([max_x, max_y, back_z ], [0., 0., -1.], [uv_right, uv_top]);
                            builder.push_indices(vec![4, 6, 5, 4, 7, 6]);
                        }
                    },
                    MaterialMode::Repeat { step, .. } => {
                        let y_over = (h / step.y) - (h / step.y).floor();
                        let x_over = (w / step.x) - (w / step.x).floor();

                        let y_steps = (h / step.y).abs().ceil() as i32;
                        for y in 0..y_steps {
                            let offset_y = step.y * y as f32 + offset.y;
                            let min_y = -extent_y + offset_y;
                            let max_y = (-extent_y + offset_y + step.y).min(extent_y + offset.y);
                            let x_steps = (w / step.x).abs().ceil() as i32;
                            for x in 0..x_steps {
                                let offset_x = step.x * x as f32 + offset.x;
                                let i = builder.len() as u32;
                                let min_x = -extent_x + offset_x;
                                let max_x = (-extent_x + offset_x + step.x).min(extent_x + offset.x);
                                let atlas_idx = mat.mode.atlas_index(atlas_idx);
                                let [uv_left, uv_right, uv_top, uv_bottom] = mat.get_uvs(atlas_idx);
                                let uvs = mat.mode.uv_rotate(x, y, uv_left, uv_right, uv_top, uv_bottom,
                                    if (x + 1) == x_steps && x_over > 0. { x_over } else { 1. },
                                    if (y + 1) == y_steps && y_over > 0. { y_over } else { 1. },
                                );
                                let front_z = offset.z + d / 2.;
                                let back_z  = offset.z - d / 2.;
                                builder.push([min_x, min_y, front_z], [0., 0., 1.], uvs[0]);
                                builder.push([min_x, max_y, front_z], [0., 0., 1.], uvs[1]);
                                builder.push([max_x, max_y, front_z], [0., 0., 1.], uvs[2]);
                                builder.push([max_x, min_y, front_z], [0., 0., 1.], uvs[3]);
                                builder.push_indices(vec![i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                                if !one_sided {
                                    builder.push([min_x, max_y, back_z ], [0., 0., -1.], uvs[1]);
                                    builder.push([min_x, min_y, back_z ], [0., 0., -1.], uvs[0]);
                                    builder.push([max_x, min_y, back_z ], [0., 0., -1.], uvs[3]);
                                    builder.push([max_x, max_y, back_z ], [0., 0., -1.], uvs[2]);
                                    builder.push_indices(vec![i + 4, i + 6, i + 5, i + 4, i + 7, i + 6]);
                                }
                            }
                        }
                    },
                }
            },
        }
        builder.build()
    }

    // pub fn make_collider(&self) {
    //     Collider::cuboid
    // }
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
    #[serde(default)]
    pub is_solid:  bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Reflect, Serialize)]
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
impl LightKind {
    pub fn default_directional() -> LightKind {
        LightKind::Directional { illuminance: default_illuminance(), length: default_length() }
    }
    pub fn default_point() -> LightKind {
        LightKind::Point { intensity: default_intensity(), range: default_range(), radius: default_radius() }
    }
    pub fn default_spotlight(target: Vec3) -> LightKind {
        LightKind::SpotLight { target, intensity: default_intensity(), range: default_range(), radius: default_radius(), inner_angle: default_inner_angle(), outer_angle: default_outer_angle() }
    }
    pub fn value(&self) -> f32 {
        match self {
            LightKind::Directional { illuminance, .. } => *illuminance,
            LightKind::Point { intensity, .. } => *intensity,
            LightKind::SpotLight { intensity, .. } => *intensity,
        }
    }
}
impl LuaUserData for LightKind {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, this| match this {
            LightKind::Directional {..} => Ok("directional".to_string()),
            LightKind::Point {..}       => Ok("point".to_string()),
            LightKind::SpotLight {..}   => Ok("spotlight".to_string()),
        });
        // Point + SpotLight
        fields.add_field_method_get("intensity", |_, this| match this {
            LightKind::Point     { intensity, .. } => Ok(Some(intensity.clone())),
            LightKind::SpotLight { intensity, .. } => Ok(Some(intensity.clone())),
            _ => Ok(None),
        });
        fields.add_field_method_set("intensity", |_, this, new_intensity: f32| match this {
            LightKind::Point { ref mut intensity, ..} => {
                *intensity = new_intensity;
                Ok(())
            },
            LightKind::SpotLight { ref mut intensity, ..} => {
                *intensity = new_intensity;
                Ok(())
            },
            _ => Err(LuaError::UserDataTypeMismatch),
        });
        fields.add_field_method_get("radius", |_, this| match this {
            LightKind::Point     { radius, .. } => Ok(Some(radius.clone())),
            LightKind::SpotLight { radius, .. } => Ok(Some(radius.clone())),
            _ => Ok(None),
        });
        fields.add_field_method_set("radius", |_, this, new_radius: f32| match this {
            LightKind::Point { ref mut radius, ..} => {
                *radius = new_radius;
                Ok(())
            },
            LightKind::SpotLight { ref mut radius, ..} => {
                *radius = new_radius;
                Ok(())
            },
            _ => Err(LuaError::UserDataTypeMismatch),
        });
        fields.add_field_method_get("range", |_, this| match this {
            LightKind::Point     { range, .. } => Ok(Some(range.clone())),
            LightKind::SpotLight { range, .. } => Ok(Some(range.clone())),
            _ => Ok(None),
        });
        fields.add_field_method_set("range", |_, this, new_range: f32| match this {
            LightKind::Point { ref mut range, ..} => {
                *range = new_range;
                Ok(())
            },
            LightKind::SpotLight { ref mut range, ..} => {
                *range = new_range;
                Ok(())
            },
            _ => Err(LuaError::UserDataTypeMismatch),
        });
        // Spotlight
        fields.add_field_method_get("target", |_, this| match this {
            LightKind::SpotLight { target, .. } => Ok(Some(LuaVec3::new(target.clone()))),
            _ => Ok(None),
        });
        fields.add_field_method_set("target", |_, this, new_target: LuaVec3| match this {
            LightKind::SpotLight { ref mut target, ..} => {
                *target = new_target.0;
                Ok(())
            },
            _ => Err(LuaError::UserDataTypeMismatch),
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
    }
}
impl LuaMod for LightKind {
    fn mod_name() -> &'static str { "LightKind" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("directional", lua.create_function(|_ctx, (illuminance, length)| {
            Ok(LightKind::Directional { illuminance, length })
        })?)?;
        table.set("default_directional", lua.create_function(|_ctx, ()| {
            Ok(LightKind::default_directional())
        })?)?;
        table.set("point", lua.create_function(|_ctx, (intensity, range, radius)| {
            Ok(LightKind::Point { intensity, range, radius })
        })?)?;
        table.set("default_point", lua.create_function(|_ctx, ()| {
            Ok(LightKind::default_point())
        })?)?;
        table.set("spotlight", lua.create_function(|_ctx, (target, intensity, range, radius, inner_angle, outer_angle)| {
            let target: LuaVec3 = target;
            Ok(LightKind::SpotLight { target: target.0, intensity, range, radius, inner_angle, outer_angle })
        })?)?;
        table.set("default_spotlight", lua.create_function(|_ctx, target: LuaVec3| {
            Ok(LightKind::default_spotlight(target.0))
        })?)?;
        Ok(())
    }
}

#[derive(Clone, Component, Copy, Debug, Deserialize, Inspectable, PartialEq, Reflect, Serialize)]
#[reflect(Component)]
pub enum LightAnim {
    Constant {
        mul: f32,
    },
    Sin {
        period: f32,
        amplitude: f32,
        #[serde(default)] 
        phase_shift: f32,
    },
}
impl Default for LightAnim {
    fn default() -> Self {
        LightAnim::Constant { mul: 1. }
    }
}
impl LuaUserData for LightAnim {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, this| match this {
            LightAnim::Constant {..} => Ok("constant".to_string()),
            LightAnim::Sin {..} => Ok("sin".to_string()),
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
    }
}
impl LuaMod for LightAnim {
    fn mod_name() -> &'static str { "LightAnim" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("constant", lua.create_function(|_ctx, mul| {
            Ok(LightAnim::Constant { mul })
        })?)?;
        table.set("sin", lua.create_function(|_ctx, (amplitude, period, phase_shift)| {
            let phase_shift: Option<f32> = phase_shift;
            Ok(LightAnim::Sin { amplitude, period, phase_shift: phase_shift.unwrap_or(0.) })
        })?)?;
        Ok(())
    }
}
#[derive(Clone, Component, Copy, Debug, Default, Deserialize, Inspectable, PartialEq, Reflect, Serialize)]
#[reflect(Component)]
pub struct LightAnimState {
    pub base_value: f32,
}

#[derive(Clone, Component, Debug, Deserialize, Reflect, Serialize)]
#[reflect(Component)]
pub struct Light {
    pub label: Option<String>,
    pub pos:   Vec3,
    pub kind:  LightKind,
    pub color: DynColor,
    #[serde(default = "default_shadows_enabled")]
    pub shadows_enabled: bool,
    #[serde(default = "default_shadow_depth_bias")]
    pub shadow_depth_bias: f32,
    #[serde(default = "default_shadow_normal_bias")]
    pub shadow_normal_bias: f32,
    #[serde(default)]
    pub anim: LightAnim,
}
fn default_color() -> DynColor { DynColor::CONST_WHITE }
fn default_shadows_enabled() -> bool { true }
pub fn default_shadow_depth_bias() -> f32 { 0.02 }
pub fn default_shadow_normal_bias() -> f32 { 0.6 }
impl Default for Light {
    fn default() -> Self {
        Light {
            label: None,
            pos:   Vec3::ZERO,
            kind:  LightKind::default_point(),
            color: default_color(),
            shadows_enabled: default_shadows_enabled(),
            shadow_depth_bias: default_shadow_depth_bias(),
            shadow_normal_bias: default_shadow_normal_bias(),
            anim: LightAnim::default(),
        }
    }
}
impl Light {
    pub fn insert(&self, commands: &mut EntityCommands, offset: Vec3) {
        match self.kind {
            LightKind::Point { intensity, range, radius } => {
                commands.insert(PointLightBundle {
                    point_light: PointLight {
                        intensity,
                        range,
                        radius,
                        color: self.color.placeholder(),
                        shadows_enabled: self.shadows_enabled,
                        shadow_depth_bias: self.shadow_depth_bias,
                        shadow_normal_bias: self.shadow_normal_bias,
                    },
                    transform: Transform::from_translation(offset + self.pos),
                    ..default()
                });
            },
            LightKind::SpotLight { target, intensity, range, radius, inner_angle, outer_angle } => {
                commands.insert(SpotLightBundle {
                    spot_light: SpotLight {
                        intensity,
                        range,
                        radius,
                        outer_angle,
                        inner_angle,
                        color: self.color.placeholder(),
                        shadows_enabled: self.shadows_enabled,
                        shadow_depth_bias: self.shadow_depth_bias,
                        shadow_normal_bias: self.shadow_normal_bias,
                    },
                    transform: Transform::from_translation(offset + self.pos).looking_at(target, Vec3::Y),
                    ..default()
                });
            },
            LightKind::Directional { illuminance, length } => {
                commands.insert(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        illuminance,
                        color: self.color.placeholder(),
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
                });
            },
        }
        commands.insert((
            self.clone(),
            self.anim,
            LightAnimState { base_value: self.kind.value() },
            SingleColored(self.color.clone()),
        ));
        if let Some(label) = &self.label {
            commands.insert(Name::new(label.clone()));
        }
    }

    pub fn insert_mut(&self, entity: &mut EntityMut, offset: Vec3) {
        match self.kind {
            LightKind::Point { intensity, range, radius } => {
                entity.insert(PointLightBundle {
                    point_light: PointLight {
                        intensity,
                        range,
                        radius,
                        color: self.color.placeholder(),
                        shadows_enabled: self.shadows_enabled,
                        shadow_depth_bias: self.shadow_depth_bias,
                        shadow_normal_bias: self.shadow_normal_bias,
                    },
                    transform: Transform::from_translation(offset + self.pos),
                    ..default()
                });
            },
            LightKind::SpotLight { target, intensity, range, radius, inner_angle, outer_angle } => {
                entity.insert(SpotLightBundle {
                    spot_light: SpotLight {
                        intensity,
                        range,
                        radius,
                        outer_angle,
                        inner_angle,
                        color: self.color.placeholder(),
                        shadows_enabled: self.shadows_enabled,
                        shadow_depth_bias: self.shadow_depth_bias,
                        shadow_normal_bias: self.shadow_normal_bias,
                    },
                    transform: Transform::from_translation(offset + self.pos).looking_at(target, Vec3::Y),
                    ..default()
                });
            },
            LightKind::Directional { illuminance, length } => {
                entity.insert(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        illuminance,
                        color: self.color.placeholder(),
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
                });
            },
        }
        entity.insert((
            self.clone(),
            self.anim,
            LightAnimState { base_value: self.kind.value() },
            SingleColored(self.color.clone()),
        ));
        if let Some(label) = &self.label {
            entity.insert(Name::new(label.clone()));
        }
    }
}
impl LuaUserData for Light {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("anim", |_, this| Ok(this.anim.clone()));
        fields.add_field_method_set("anim", |_, this, anim: LightAnim| {
            this.anim = anim;
            Ok(())
        });
        fields.add_field_method_get("color", |_, this| Ok(this.color.clone()));
        fields.add_field_method_set("color", |_, this, any: Any3<DynColor, RgbaColor, String>| Ok(match any {
            Any3::A(c) => { this.color = c },
            Any3::B(c) => { this.color = DynColor::Custom(c) },
            Any3::C(c) => { this.color = DynColor::Named(c) },
        }));
        fields.add_field_method_get("label", |_, this| Ok(this.label.clone()));
        fields.add_field_method_set("label", |_, this, label: Option<String>| {
            this.label = label;
            Ok(())
        });
        fields.add_field_method_get("pos", |_, this| Ok(LuaVec3::new(this.pos.clone())));
        fields.add_field_method_set("pos", |_, this, pos: LuaVec3| {
            this.pos = pos.0;
            Ok(())
        });
        fields.add_field_method_get("shadow_depth_bias", |_, this| Ok(this.shadow_depth_bias.clone()));
        fields.add_field_method_set("shadow_depth_bias", |_, this, shadow_depth_bias: f32| {
            this.shadow_depth_bias = shadow_depth_bias;
            Ok(())
        });
        fields.add_field_method_get("shadows_enabled", |_, this| Ok(this.shadows_enabled.clone()));
        fields.add_field_method_set("shadows_enabled", |_, this, shadows_enabled: bool| {
            this.shadows_enabled = shadows_enabled;
            Ok(())
        });
        fields.add_field_method_get("shadow_normal_bias", |_, this| Ok(this.shadow_normal_bias.clone()));
        fields.add_field_method_set("shadow_normal_bias", |_, this, shadow_normal_bias: f32| {
            this.shadow_normal_bias = shadow_normal_bias;
            Ok(())
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_method("spawn", |ctx, this, ()| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let mut write = world.write();
            let mut entity = write.spawn_empty();
            this.insert_mut(&mut entity, Vec3::ZERO);
            Ok(LuaEntity::new(entity.id()))
        });
        methods.add_method("apply", |ctx, this, entity: LuaEntity| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let mut write = world.write();
            let mut entity = write.entity_mut(entity.0);
            this.insert_mut(&mut entity, Vec3::ZERO);
            Ok(LuaEntity::new(entity.id()))
        });
    }
}
impl LuaMod for Light {
    fn mod_name() -> &'static str { "Light" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("of", lua.create_function(|ctx, entity: LuaEntity| {
            if let Some(ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity(entity.0) {
                Ok(ent.get::<Light>().cloned())
            } else { Ok(None) }
        })?)?;
        table.set("new", lua.create_function(|_ctx, kind: LightKind| {
            Ok(Light {
                label: None,
                pos:   Vec3::ZERO,
                kind,
                color: DynColor::Named("white".to_string()),
                shadows_enabled: default_shadows_enabled(),
                shadow_depth_bias: default_shadow_depth_bias(),
                shadow_normal_bias: default_shadow_normal_bias(),
                anim: LightAnim::default(),
            })
        })?)?;
        Ok(())
    }
}