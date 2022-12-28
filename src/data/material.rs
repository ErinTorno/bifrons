use std::{path::{PathBuf}, collections::{HashMap, HashSet}};

use bevy::{prelude::*, render::{render_resource::{AddressMode, SamplerDescriptor, FilterMode}, texture::ImageSampler}};
use bevy_inspector_egui::prelude::*;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{scripting::{LuaMod, color::RgbaColor, bevy_api::{LuaEntity, handle::LuaHandle}}};

use super::{lua::{LuaWorld, Any3}, palette::{DynColor}};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum RepeatType {
    Identity,
    Rotate180,
}
impl RepeatType {
    // pub fn map_uvs(&self, x: i32, y: i32, uvs: [f32; 4]) -> [f32; 4] {
    //     match self {
    //         RepeatType::Rotate180 if (x + y) % 2 == 0 => [uv_right, uv_left, uv_bottom, uv_top],
    //         _ => uvs,
    //     };
    // }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MaterialMode {
    Stretch,
    Repeat {
        #[serde(default = "default_step")]
        step:    f32,
        #[serde(default = "default_on_step")]
        on_step: RepeatType,
    },
}
fn default_step() -> f32 { 1. }
fn default_on_step() -> RepeatType { RepeatType::Identity }
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
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(match this {
            MaterialMode::Stretch => "MatMode.stretch()".to_string(),
            MaterialMode::Repeat { step, on_step }=> format!("MatMode.repeat{{step = {}, on_step = {}}}", step, format!("{:?}", on_step).to_lowercase()),
        }));
    }
}
impl LuaMod for MaterialMode {
    fn mod_name() -> &'static str { "MatMode" }
    fn register_defs(lua: &Lua, table: &mut LuaTable<'_>) -> Result<(), LuaError> {
        table.set("stretch", lua.create_function(|_ctx, ()| Ok(MaterialMode::Stretch))?)?;
        table.set("repeat", lua.create_function(|_ctx, table: LuaTable| {
            let step = if table.contains_key("step")? {
                table.get::<_, f32>("step")?
            } else { default_step() };
            let on_step = if table.contains_key("mode")? {
                let mode = table.get::<_, String>("mode")?;
                match mode.as_str() {
                    "identity" => RepeatType::Identity,
                    "rotate180" => RepeatType::Rotate180,
                    _ => {
                        return Err(LuaError::RuntimeError(format!("No known RepeatType \"{}\"; valid values are {{identity, rotate180}}", mode)));
                    }
                }
            } else { default_on_step() };
            Ok(MaterialMode::Repeat { step, on_step })
        })?)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AtlasIndex {
    pub row: f32,
    pub col: f32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Atlas {
    pub rows:    f32,
    pub columns: f32,
    pub width:   f32,
    pub height:  f32,
}
impl LuaUserData for Atlas {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("rows", |_, this| Ok(this.rows));
        fields.add_field_method_set("rows", |_, this, rows| Ok(this.rows = rows));
        fields.add_field_method_get("columns", |_, this| Ok(this.columns));
        fields.add_field_method_set("columns", |_, this, columns| Ok(this.columns = columns));
        fields.add_field_method_get("width", |_, this| Ok(this.width));
        fields.add_field_method_set("width", |_, this, width| Ok(this.width = width));
        fields.add_field_method_get("height", |_, this| Ok(this.height));
        fields.add_field_method_set("height", |_, this, height| Ok(this.height = height));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("Atlas{{rows = {}, columns = {}, width = {}, height = {}}}", this.rows, this.columns, this.width, this.height)));
    }
}
impl LuaMod for Atlas {
    fn mod_name() -> &'static str { "Atlas" }
    fn register_defs(lua: &Lua, table: &mut LuaTable<'_>) -> Result<(), LuaError> {
        table.set("new", lua.create_function(|_ctx, (rows, columns, width, height)| Ok(Atlas { rows, columns, width, height}))?)?;
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
    #[serde(default = "default_alpha_blend")]
    pub alpha_blend:      bool,
    #[serde(default)]
    pub unlit:            bool,
}
fn default_emissive_color() -> DynColor { DynColor::CONST_BLACK }
fn default_metallic() -> f32 { 0.01 }
fn default_reflectance() -> f32 { 0.5 }
fn default_alpha_blend() -> bool { false }
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
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        let handles = self.load_textures(&asset_server);
        let handle = materials.add(self.make_material(handles, tex_mat_info));
        tex_mat_info.materials.insert(handle.clone(), self.clone());
        handle
    }

    pub fn make_material(&self, handles: TextureHandles, tex_mat_info: &mut TexMatInfo) -> StandardMaterial {
        let address_mode = self.mode.into();
        tex_mat_info.samplers.insert(handles.texture.clone(), ImageSampler::Descriptor(SamplerDescriptor {
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            ..default()
        }));
        StandardMaterial {
            base_color_texture: Some(handles.texture),
            emissive_texture: handles.emissive,
            normal_map_texture: handles.normal,
            alpha_mode: if self.alpha_blend { AlphaMode::Blend } else { AlphaMode::Mask(0.05) }, // hack-hack until Bevy bug with AlphaMode::Blend render orders
            unlit: self.unlit || self.color == DynColor::Background,
            metallic: self.metallic,
            reflectance: self.reflectance,
            // double_sided: true,
            // cull_mode: None,
            ..default()
        }
    }

    pub fn get_uvs(&self, idx: AtlasIndex) -> [f32; 4] {
        if let Some(atlas) = self.atlas {
            let uv_left   = idx.col / atlas.columns;
            let uv_right  = (idx.col + 1.) / atlas.columns;
            let uv_top    = idx.row / atlas.rows;
            let uv_bottom = (idx.row + 1.) / atlas.rows;
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
        alpha_blend: false,
        unlit: true,
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
        alpha_blend: false,
        unlit: false,
    };
}
impl LuaUserData for TextureMaterial {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
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
        fields.add_field_method_get("metallic", |_, this| Ok(this.metallic));
        fields.add_field_method_set("metallic", |_, this, metallic| Ok(this.metallic = metallic));
        fields.add_field_method_get("reflectance", |_, this| Ok(this.reflectance));
        fields.add_field_method_set("reflectance", |_, this, reflectance| Ok(this.reflectance = reflectance));
        fields.add_field_method_get("alpha_blend", |_, this| Ok(this.alpha_blend));
        fields.add_field_method_set("alpha_blend", |_, this, alpha_blend| Ok(this.alpha_blend = alpha_blend));
        fields.add_field_method_get("texture", |_, this| Ok(this.texture.clone()));
        fields.add_field_method_set("texture", |_, this, texture| Ok(this.texture = texture));
        fields.add_field_method_get("emissive_texture", |_, this| Ok(this.emissive_texture.clone()));
        fields.add_field_method_set("emissive_texture", |_, this, emissive_texture| Ok(this.emissive_texture = emissive_texture));
        fields.add_field_method_get("normal_texture", |_, this| Ok(this.normal_texture.clone()));
        fields.add_field_method_set("normal_texture", |_, this, normal_texture| Ok(this.normal_texture = normal_texture));
        fields.add_field_method_get("unlit", |_, this| Ok(this.unlit));
        fields.add_field_method_set("unlit", |_, this, unlit| Ok(this.unlit = unlit));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_method("add_to_assets", |ctx, this, ()| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
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
        });
        methods.add_method("apply", |ctx, this, mat_handle: LuaHandle| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
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
        table.set("handle_of", lua.create_function(|ctx, entity: LuaEntity| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            if let Some(handle) = w.get::<Handle<StandardMaterial>>(entity.0) {
                Ok(Some(LuaHandle::from(handle.clone())))
            } else { Ok(None) }
        })?)?;
        table.set("handle_table", lua.create_function(|ctx, entity: LuaEntity| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let entity = entity.0;
            if let Some(mats) = w.get::<LoadedMaterials>(entity) {
                let table = ctx.create_table()?;
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