use std::{path::{PathBuf}, collections::{HashMap, HashSet}, f32::consts::PI};

use bevy::{prelude::*, render::{render_resource::{AddressMode, SamplerDescriptor, FilterMode}, texture::ImageSampler}};
use bevy_inspector_egui::prelude::*;
use mlua::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{scripting::{LuaMod, bevy_api::{LuaEntity, handle::LuaHandle, math::LuaVec2}}};

use super::{lua::{LuaWorld, Any3, Any2}, palette::{DynColor}, rgba::RgbaColor};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum RepeatOp {
    AtlasRandom(Vec<AtlasIndex>),
    Identity,
    Rotate       { quarters: i32 },
    RotateRandom { quarters: i32 },
}
impl RepeatOp {
    pub fn uv_rotate(&self, x: i32, y: i32) -> Vec2 {
        let rot_z = match self {
            RepeatOp::Rotate { quarters } => ((x + y) * quarters).rem_euclid(4),
            RepeatOp::RotateRandom { quarters } => (rand::thread_rng().gen_range(0..4) * quarters).rem_euclid(4),
            _ => 0,
        } as f32;
        Vec2::from_angle(PI * 0.5 * rot_z)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum MaterialMode {
    Stretch,
    Repeat {
        #[serde(default = "default_step")]
        step:    Vec2,
        #[serde(default = "default")]
        on_step: Vec<RepeatOp>,
    },
}
fn default_step() -> Vec2 { Vec2::ONE }
impl MaterialMode {
    pub fn atlas_index(&self, def: AtlasIndex) -> AtlasIndex {
        match &self {
            MaterialMode::Stretch => def,
            MaterialMode::Repeat { on_step, .. } => {
                for op in on_step.iter() {
                    if let RepeatOp::AtlasRandom(indices) = op {
                        if !indices.is_empty() {
                            let i = rand::thread_rng().gen_range(0..indices.len());
                            return indices[i];
                        }
                    }
                }
                def
            },
        }
    }

    pub fn uv_rotate(&self, x: i32, y: i32, uv_left: f32, uv_right: f32, uv_top: f32, uv_bottom: f32, x_over: f32, y_over: f32) -> [[f32; 2]; 4] {
        let center   = Vec2::new((uv_right - uv_left) / 2. + uv_left, (uv_bottom - uv_top) / 2. + uv_top);
        let uv_right = x_over * (uv_right - uv_left) + uv_left;
        let uv_top   = uv_bottom - y_over * (uv_bottom - uv_top);
        let uvs = [
            [uv_left,  uv_bottom],
            [uv_left,  uv_top],
            [uv_right, uv_top],
            [uv_right, uv_bottom],
        ];
        match self {
            MaterialMode::Stretch => uvs,
            MaterialMode::Repeat { on_step, .. } => {
                let rotation = on_step.iter()
                    .map(|op| op.uv_rotate(x, y))
                    .fold(Vec2::from_angle(0.), |a, b| b.rotate(a));
                fn rotate(rotation: Vec2, center: Vec2, uv: [f32; 2]) -> [f32; 2] {
                    let v = Vec2::new(uv[0], uv[1]) - center;
                    let v = rotation.rotate(v) + center;
                    [v.x, v.y]
                }
                [
                    rotate(rotation, center, uvs[0]),
                    rotate(rotation, center, uvs[1]),
                    rotate(rotation, center, uvs[2]),
                    rotate(rotation, center, uvs[3]),
                ]
            },
        }
    }
}
impl Default for MaterialMode {
    fn default() -> Self { MaterialMode::Stretch }
}
impl Into<AddressMode> for MaterialMode {
    fn into(self) -> AddressMode {
        match self {
            MaterialMode::Stretch     => AddressMode::ClampToEdge,
            MaterialMode::Repeat {..} => AddressMode::Repeat,
        }
    }
}
impl LuaUserData for MaterialMode {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: MaterialMode| Ok(this == &that));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(match this {
            MaterialMode::Stretch => "MatMode.stretch()".to_string(),
            MaterialMode::Repeat { step, on_step }=> format!("MatMode.repeat{{step = {}, on_step = \"{}\"}}", step, format!("{:?}", on_step).to_lowercase()),
        }));
    }
}
impl LuaMod for MaterialMode {
    fn mod_name() -> &'static str { "MatMode" }
    fn register_defs(lua: &Lua, table: &mut LuaTable<'_>) -> Result<(), LuaError> {
        table.set("stretch", MaterialMode::Stretch.to_lua(lua)?)?;
        table.set("repeat", lua.create_function(|_, table: LuaTable| {
            let step = table.get::<_, Option<LuaVec2>>("step")?.map(|v| v.0).unwrap_or(default_step());
            let on_step = if let Some(ops_table) = table.get::<_, Option<LuaTable>>("ops")? {
                let mut ops = Vec::new();
                for r in ops_table.pairs::<LuaValue, LuaTable>() {
                    let (_, op_config) = r?;
                    let name = op_config.get::<_, String>("op")?;
                    ops.push(match name.as_str() {
                        "atlasrandom"   => {
                            let mut v = Vec::new();
                            for i in op_config.sequence_values::<AtlasIndex>() {
                                v.push(i?);
                            }
                            RepeatOp::AtlasRandom(v)
                        },
                        "identity" => RepeatOp::Identity,
                        "rotate"   => {
                            let quarters = op_config.get("quarters")?;
                            RepeatOp::Rotate { quarters }
                        },
                        "rotaterandom"   => {
                            let quarters = op_config.get("quarters")?;
                            RepeatOp::Rotate { quarters }
                        },
                        _ => {
                            return Err(mlua::Error::RuntimeError(format!("Unknown matmode op {:?}", name)))
                        }
                    });
                }
                ops
            } else { Vec::new() };
            Ok(MaterialMode::Repeat { step, on_step })
        })?)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AtlasIndex {
    pub col: f32,
    pub row: f32,
}
impl<'lua> FromLua<'lua> for AtlasIndex {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        Ok(match Any2::<LuaTable, LuaVec2>::from_lua(lua_value, lua)? {
            Any2::A(table)      => AtlasIndex { col: table.get("col")?, row: table.get("row")? },
            Any2::B(LuaVec2(v)) => AtlasIndex { col: v.x, row: v.y },
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Atlas {
    pub width:   f32,
    pub height:  f32,
    #[serde(default)]
    pub offset:  Vec2,
}
impl LuaUserData for Atlas {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("width",  |_, this| Ok(this.width));
        fields.add_field_method_set("width",  |_, this, width| Ok(this.width = width));
        fields.add_field_method_get("height", |_, this| Ok(this.height));
        fields.add_field_method_set("height", |_, this, height| Ok(this.height = height));
        fields.add_field_method_get("offset", |_, this| Ok(LuaVec2(this.offset)));
        fields.add_field_method_set("offset", |_, this, v: LuaVec2| Ok(this.offset = v.0));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: Atlas| Ok(this == &that));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("atlas{{width = {}, height = {}}}", this.width, this.height)));
    }
}
impl LuaMod for Atlas {
    fn mod_name() -> &'static str { "Atlas" }
    fn register_defs(lua: &Lua, table: &mut LuaTable<'_>) -> Result<(), LuaError> {
        table.set("new", lua.create_function(|_, (width, height, offset): (_, _, Option<LuaVec2>)| Ok(Atlas { width, height, offset: offset.map(|v| v.0).unwrap_or(Vec2::ZERO)}))?)?;
        Ok(())
    }
}

#[derive(Clone, Component, Debug, Default, Inspectable)]
pub struct LoadedMaterials {
    pub by_name: HashMap<String, Handle<StandardMaterial>>,
}

pub fn resolve_texture_path<S>(file: S, asset_server: &AssetServer) -> PathBuf where S: Into<PathBuf> {
    let path: PathBuf = file.into();
    if let Some("png") = path.extension().and_then(|s| s.to_str()) {
        let mut ktx2 = path.clone();
        ktx2.set_extension("ktx2");
        if asset_server.asset_io().is_file(&ktx2) {
            return ktx2;
        }
    }
    path
}

pub fn resolve_and_load_texture(file: &String, asset_server: &AssetServer) -> Handle<Image> {
    let path = resolve_texture_path(file, asset_server);
    asset_server.load(path)
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum TextureFilter {
    Nearest,
    Linear
}
impl From<TextureFilter> for FilterMode {
    fn from(value: TextureFilter) -> Self {
        match value {
            TextureFilter::Nearest => FilterMode::Nearest,
            TextureFilter::Linear  => FilterMode::Linear,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextureMaterial {
    pub texture:          String,
    #[serde(default)]
    pub normal_texture:   Option<String>,
    #[serde(default)]
    pub emissive_texture: Option<String>,
    #[serde(default = "default_emissive_color")]
    pub emissive_color:   DynColor,
    #[serde(default)]
    pub atlas:            Option<Atlas>,
    #[serde(default)]
    pub mode:             MaterialMode,
    pub color:            DynColor,
    #[serde(default = "default_metallic")]
    pub metallic:         f32,
    #[serde(default = "default_reflectance")]
    pub reflectance:      f32,
    #[serde(default)]
    pub alpha_blend:      Option<bool>,
    #[serde(default)]
    pub unlit:            bool,
    #[serde(default = "default_filter")]
    pub filter:           TextureFilter,
}
fn default_emissive_color() -> DynColor { DynColor::CONST_BLACK }
fn default_metallic() -> f32 { 0.01 }
fn default_reflectance() -> f32 { 0.25 }
fn default_filter() -> TextureFilter { TextureFilter::Linear }
impl TextureMaterial {
    pub fn load_textures(&self, asset_server: &AssetServer) -> TextureHandles {
        TextureHandles {
            texture:  resolve_and_load_texture(&self.texture, asset_server),
            normal:   self.normal_texture.as_ref().map(|p| resolve_and_load_texture(p, asset_server)),
            emissive: self.emissive_texture.as_ref().map(|p| resolve_and_load_texture(p, asset_server)),
        }
    }

    pub fn load_material(
        &self,
        asset_server: &AssetServer,
        tex_mat_info: &mut TexMatInfo,
        materials:    &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        let handles = self.load_textures(&asset_server);
        let handle = materials.add(self.make_material(handles, tex_mat_info));
        tex_mat_info.materials.insert(handle.clone(), self.clone());
        handle
    }

    pub fn make_material(&self, handles: TextureHandles, tex_mat_info: &mut TexMatInfo) -> StandardMaterial {
        let address_mode = self.mode.clone().into();
        tex_mat_info.samplers.insert(handles.texture.clone(), ImageSampler::Descriptor(SamplerDescriptor {
            mag_filter:     self.filter.into(),
            min_filter:     self.filter.into(),
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            ..default()
        }));
        StandardMaterial {
            base_color_texture: Some(handles.texture),
            emissive_texture:   handles.emissive,
            normal_map_texture: handles.normal,
            alpha_mode:         if self.alpha_blend.unwrap_or(self.color != DynColor::Background) { AlphaMode::Blend } else { AlphaMode::Mask(0.05) }, // hack-hack until Bevy bug with AlphaMode::Blend render orders
            unlit:              self.unlit || self.color == DynColor::Background,
            metallic:           self.metallic,
            reflectance:        self.reflectance,
            // double_sided: true,
            // cull_mode: None,
            ..default()
        }
    }

    pub fn get_uvs(&self, idx: AtlasIndex) -> [f32; 4] {
        if let Some(atlas) = self.atlas {
            let uv_left   = atlas.offset.x + atlas.width * idx.col;
            let uv_right  = atlas.offset.x + atlas.width * (idx.col + 1.);
            let uv_top    = atlas.offset.y + atlas.height * idx.row;
            let uv_bottom = atlas.offset.y + atlas.height * (idx.row + 1.);
            [uv_left, uv_right, uv_top, uv_bottom]
        } else { [0., 1., 0., 1.] }
    }

    pub const BACKGROUND: TextureMaterial = TextureMaterial {
        texture: String::new(),
        normal_texture: None,
        mode: MaterialMode::Stretch,
        color: DynColor::Background,
        emissive_color: DynColor::CONST_BLACK,
        emissive_texture: None,
        metallic: 0.,
        reflectance: 0.,
        atlas: None,
        alpha_blend: Some(false),
        unlit: true,
        filter: TextureFilter::Nearest,
    };
    pub const MISSING: TextureMaterial = TextureMaterial {
        texture: String::new(),
        normal_texture: None,
        mode: MaterialMode::Stretch,
        color:          DynColor::CONST_FUCHSIA,
        emissive_color: DynColor::CONST_BLACK,
        emissive_texture: None,
        metallic: 0.6,
        reflectance: 0.8,
        atlas: None,
        alpha_blend: Some(false),
        unlit: false,
        filter: TextureFilter::Nearest,
    };
}
impl LuaUserData for TextureMaterial {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("alpha_blend", |_, this| Ok(this.alpha_blend));
        fields.add_field_method_set("alpha_blend", |_, this, alpha_blend| Ok(this.alpha_blend = alpha_blend));
        fields.add_field_method_get("atlas", |_, this| Ok(this.atlas.clone()));
        fields.add_field_method_set("atlas", |_, this, atlas| Ok(this.atlas = atlas));
        fields.add_field_method_get("color", |_, this| Ok(this.color.clone()));
        fields.add_field_method_set("color", |_, this, any: Any3<DynColor, RgbaColor, String>| Ok(match any {
            Any3::A(c) => { this.color = c },
            Any3::B(c) => { this.color = DynColor::Custom(c) },
            Any3::C(c) => { this.color = DynColor::Named(c) },
        }));
        fields.add_field_method_get("emissive_color", |_, this| Ok(this.emissive_color.clone()));
        fields.add_field_method_set("emissive_color", |_, this, any: Any3<DynColor, RgbaColor, String>| Ok(match any {
            Any3::A(c) => { this.emissive_color = c },
            Any3::B(c) => { this.emissive_color = DynColor::Custom(c) },
            Any3::C(c) => { this.emissive_color = DynColor::Named(c) },
        }));
        fields.add_field_method_get("emissive_texture", |_, this| Ok(this.emissive_texture.clone()));
        fields.add_field_method_set("emissive_texture", |_, this, emissive_texture| Ok(this.emissive_texture = emissive_texture));
        fields.add_field_method_get("metallic", |_, this| Ok(this.metallic));
        fields.add_field_method_set("metallic", |_, this, metallic| Ok(this.metallic = metallic));
        fields.add_field_method_get("reflectance", |_, this| Ok(this.reflectance));
        fields.add_field_method_set("reflectance", |_, this, reflectance| Ok(this.reflectance = reflectance));
        fields.add_field_method_get("texture", |_, this| Ok(this.texture.clone()));
        fields.add_field_method_set("texture", |_, this, texture| Ok(this.texture = texture));
        fields.add_field_method_get("mode", |_, this| Ok(this.mode.clone()));
        fields.add_field_method_set("mode", |_, this, mode| Ok(this.mode = mode));
        fields.add_field_method_get("normal_texture", |_, this| Ok(this.normal_texture.clone()));
        fields.add_field_method_set("normal_texture", |_, this, normal_texture| Ok(this.normal_texture = normal_texture));
        fields.add_field_method_get("unlit", |_, this| Ok(this.unlit));
        fields.add_field_method_set("unlit", |_, this, unlit| Ok(this.unlit = unlit));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_method("apply", |lua, this, mat_handle: LuaHandle| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let tex_handles = {
                let asset_server = w.resource::<AssetServer>();
                this.load_textures(asset_server)
            };

            let mat = {
                let mut tex_mat_info = w.resource_mut::<TexMatInfo>();
                this.make_material(tex_handles, &mut tex_mat_info)
            };
            let handle = mat_handle.handle.clone().typed_weak();
            {
                let mut materials = w.resource_mut::<Assets<StandardMaterial>>();
                if let Some(cur) = materials.get_mut(&handle) {
                    *cur = mat;
                } else {
                    return Err(LuaError::RuntimeError(format!("No material was found associated with handle {:?}", handle)));
                }
            }
            {
                let mut mats_to_init = w.resource_mut::<MaterialsToInit>();
                mats_to_init.0.insert(handle.clone_weak());
            }
            {
                let mut material_colors = w.resource_mut::<MaterialColors>();
                if let Some(loaded) = material_colors.by_handle.get_mut(&handle) {
                    loaded.tex_mat = this.clone();
                } else {
                    material_colors.by_handle.insert(handle.clone_weak(), LoadedMat { handle: handle.clone_weak(), tex_mat: this.clone() });
                }
            }
            Ok(())
        });
    }
}
impl LuaMod for TextureMaterial {
    fn mod_name() -> &'static str { "Material" }
    fn register_defs(lua: &Lua, table: &mut LuaTable<'_>) -> Result<(), LuaError> {
        table.set("add_asset", lua.create_function(|lua, this: TextureMaterial| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let tex_handles = {
                let asset_server = w.get_resource::<AssetServer>();
                if asset_server.is_none() { return Err(LuaError::RuntimeError(format!("Unable to get AssetServer"))); }
                this.load_textures(asset_server.unwrap())
            };
            let mat = {
                let tex_mat_info = w.get_resource_mut::<TexMatInfo>();
                if tex_mat_info.is_none() { return Err(LuaError::RuntimeError(format!("Unable to get TexMatInfo"))); }
                let mut tex_mat_info = tex_mat_info.unwrap();
                this.make_material(tex_handles, &mut tex_mat_info)
            };
            let materials = w.get_resource_mut::<Assets<StandardMaterial>>();
            if materials.is_none() { return Err(LuaError::RuntimeError(format!("Unable to get Assets<StandardMaterial>"))); }
            let mut materials = materials.unwrap();
            Ok(LuaHandle::from(materials.add(mat)))
        })?)?;
        table.set("handle_of", lua.create_function(|lua, entity: LuaEntity| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            if let Some(handle) = w.get::<Handle<StandardMaterial>>(entity.0) {
                Ok(Some(LuaHandle::from(handle.clone())))
            } else { Ok(None) }
        })?)?;
        table.set("handle_table", lua.create_function(|lua, entity: LuaEntity| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let entity = entity.0;
            if let Some(mats) = w.get::<LoadedMaterials>(entity) {
                let table = lua.create_table()?;
                for (name, handle) in mats.by_name.iter() {
                    table.set(name.as_str(), LuaHandle::from(handle.clone()))?;
                }
                Ok(Some(table))
            } else { Ok(None) }
        })?)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextureHandles {
    pub texture:  Handle<Image>,
    pub normal:   Option<Handle<Image>>,
    pub emissive: Option<Handle<Image>>,
}

#[derive(Clone, Debug, Default, Resource)]
pub struct TexMatInfo {
    pub materials: HashMap<Handle<StandardMaterial>, TextureMaterial>,
    pub samplers:  HashMap<Handle<Image>, ImageSampler>,
}

#[derive(Clone, Debug)]
pub struct LoadedMat {
    pub handle:  Handle<StandardMaterial>,
    pub tex_mat: TextureMaterial,
}

#[derive(Clone, Debug, Default, Resource)]
pub struct MaterialColors {
    pub by_handle: HashMap<Handle<StandardMaterial>, LoadedMat>,
}

#[derive(Clone, Debug, Default, Resource)]
pub struct MaterialsToInit(pub HashSet<Handle<StandardMaterial>>);