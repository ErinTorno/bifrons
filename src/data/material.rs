use bevy::{prelude::*, render::{render_resource::{AddressMode, SamplerDescriptor, FilterMode}, texture::ImageSampler}};
use serde::{Deserialize, Serialize};

use crate::{util::serialize::*, system::texture::ImageDescriptions};

use super::anim::{ColorLayer};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum RepeatType {
    Identity,
    Rotate180,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MaterialMode {
    Stretch,
    Repeat {
        step:    f32,
        #[serde(default = "default_on_step")]
        on_step: RepeatType,
    },
}
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextureMaterial {
    #[serde(default)]
    pub layer:   ColorLayer,
    pub texture: String,
    #[serde(default)]
    pub normal:  Option<String>,
    #[serde(default)]
    pub emissive_texture: Option<String>,
    #[serde(default = "default_emissive_color", deserialize_with = "deserialize_hex_color", serialize_with = "serialize_hex_color")]
    pub emissive_color: Color,
    #[serde(default)]
    pub atlas:   Option<Atlas>,
    #[serde(default)]
    pub mode:    MaterialMode,
    #[serde(default = "default_color", deserialize_with = "deserialize_hex_color", serialize_with = "serialize_hex_color")]
    pub color:   Color,
    #[serde(default = "default_metallic")]
    pub metallic:    f32,
    #[serde(default = "default_reflectance")]
    pub reflectance: f32,
    #[serde(default = "default_alpha_blend")]
    pub alpha_blend: bool,
}
fn default_color() -> Color { Color::WHITE }
fn default_emissive_color() -> Color { Color::BLACK }
fn default_metallic() -> f32 { 0.5 }
fn default_reflectance() -> f32 { 0.5 }
fn default_alpha_blend() -> bool { false }
impl TextureMaterial {
    pub fn load_textures(&self, asset_server: &AssetServer) -> TextureHandles {
        // todo; if .kt2 or .basis file exists at same path/name but different extension, load that instead
        TextureHandles {
            texture:  asset_server.load(&self.texture),
            normal:   self.normal.as_ref().map(|p| asset_server.load(p)),
            emissive: self.emissive_texture.as_ref().map(|p| asset_server.load(p)),
        }
    }

    pub fn make_material(&self, asset_server: &AssetServer, descriptions: &mut ImageDescriptions) -> StandardMaterial {
        let handles = self.load_textures(&asset_server);
        let address_mode = self.mode.into();
        descriptions.map.insert(handles.texture.clone(), ImageSampler::Descriptor(SamplerDescriptor {
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            ..default()
        }));
        StandardMaterial {
            base_color_texture: Some(handles.texture),
            base_color: self.color,
            emissive: self.emissive_color,
            emissive_texture: handles.emissive,
            normal_map_texture: handles.normal,
            alpha_mode: if self.alpha_blend { AlphaMode::Blend } else { AlphaMode::Mask(0.05) }, // hack-hack until Bevy bug with AlphaMode::Blend render orders
            unlit: self.layer == ColorLayer::Background,
            metallic: self.metallic,
            reflectance: self.reflectance,
            double_sided: true,
            cull_mode: None,
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
        layer: ColorLayer::Background,
        texture: String::new(),
        normal: None,
        mode: MaterialMode::Stretch,
        color: Color::WHITE,
        emissive_color: Color::BLACK,
        emissive_texture: None,
        metallic: 0.,
        reflectance: 0.,
        atlas: None,
        alpha_blend: false,
    };
    pub const MISSING: TextureMaterial = TextureMaterial {
        layer: ColorLayer::NoChange,
        texture: String::new(),
        normal: None,
        mode: MaterialMode::Stretch,
        color:          Color::rgb(1., 0., 0.615),
        emissive_color: Color::BLACK,
        emissive_texture: None,
        metallic: 0.6,
        reflectance: 0.8,
        atlas: None,
        alpha_blend: false,
    };
}

#[derive(Clone, Debug, Default)]
pub struct TextureHandles {
    pub texture:  Handle<Image>,
    pub normal:   Option<Handle<Image>>,
    pub emissive: Option<Handle<Image>>,
}