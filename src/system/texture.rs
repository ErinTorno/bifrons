
use std::collections::HashMap;

use bevy::{prelude::*, render::{texture::ImageSampler, render_resource::{SamplerDescriptor, FilterMode, Extent3d, TextureDimension, TextureFormat}}, asset::LoadState};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

use crate::data::{anim::ColorLayer, material::{TexMatInfo, LoadedMaterials}};

#[derive(Clone, Debug, Default)]
pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_inspectable::<LoadedMaterials>()
            .init_resource::<MaterialColors>()
            .init_resource::<Background>()
            .init_resource::<ImagesToCheck>()
            .init_resource::<MissingTexture>()
            .init_resource::<TexMatInfo>()
            .add_startup_system(setup_default_textures)
            .add_system(update_image_descriptor)
            .add_system(update_material_colors)
            .add_system(log_asset_errors)
        ;
    }
}

#[derive(Clone, Debug, Default)]
pub struct MaterialColors {
    pub layers: HashMap<Handle<StandardMaterial>, ColorLayer>,
}

pub fn update_material_colors(
    mats:            Res<MaterialColors>,
    clear_color:     Res<ClearColor>,
    mut background:  ResMut<Background>,
    mut materials:   ResMut<Assets<StandardMaterial>>,
) {
    if background.color != clear_color.0 {
        if let Some(mat) = materials.get_mut(&background.material) {
            background.color = clear_color.0;
            mat.base_color = clear_color.0.clone();
        }
    }
    for (handle, layer) in mats.layers.iter() {
        match *layer {
            ColorLayer::Background => {
                if let Some(mat) = materials.get_mut(handle) {
                    mat.base_color = clear_color.0;
                }
            },
            ColorLayer::NoChange   => (),
            ColorLayer::Outline    => (),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ImagesToCheck {
    pub vec: Vec<Handle<Image>>,
}

pub fn get_default_sampler() -> ImageSampler {
    ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        ..default()
    })
}

pub fn log_asset_errors(
    asset_server: Res<AssetServer>,
    mut to_check: ResMut<ImagesToCheck>,
) {
    for handle in &to_check.vec {
        match asset_server.get_load_state(handle.clone()) {
            LoadState::Failed => warn!("Image {:?} failed to load", handle),
            _ => (),
        }
    }
    to_check.vec = to_check.vec.iter().filter(|handle| match asset_server.get_load_state(handle.clone()) {
        LoadState::Loaded | LoadState::Failed => false,
        _ => true,
    }).cloned().collect();
}

pub fn update_image_descriptor(
    tex_mat_info: Res<TexMatInfo>,
    mut to_check: ResMut<ImagesToCheck>,
    mut images: ResMut<Assets<Image>>,
    mut events: EventReader<AssetEvent<Image>>,
) {
    let default_sampler = get_default_sampler();
    for e in events.iter() {
        match e {
            AssetEvent::Created { handle } => {
                if let Some(image) = images.get_mut(handle) {
                    if let Some(sampler) = tex_mat_info.samplers.get(handle) {
                        image.sampler_descriptor = sampler.clone();
                    } else {
                        image.sampler_descriptor = default_sampler.clone();
                    }
                    to_check.vec.push(handle.clone());
                }
            },
            AssetEvent::Modified { handle } => {
                if let Some(image) = images.get_mut(handle) {
                    if let Some(sampler) = tex_mat_info.samplers.get(handle) {
                        image.sampler_descriptor = sampler.clone();
                    } else {
                        image.sampler_descriptor = default_sampler.clone();
                    }
                }
            },
            _ => (),
        }
    }
}


#[derive(Clone, Debug, Default)]
pub struct Background {
    pub color:    Color,
    pub material: Handle<StandardMaterial>,
}

#[derive(Clone, Debug, Default)]
pub struct MissingTexture {
    pub image:    Handle<Image>,
    pub material: Handle<StandardMaterial>,
}

pub fn setup_default_textures(
    mut background:  ResMut<Background>,
    mut missing_tex: ResMut<MissingTexture>,
    mut images:      ResMut<Assets<Image>>,
    mut materials:   ResMut<Assets<StandardMaterial>>,
) {
    background.material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..default()
    });
    background.color = Color::BLACK;

    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    let mut image = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    );
    if let ImageSampler::Descriptor(desc) =  &mut image.sampler_descriptor {
        desc.min_filter = FilterMode::Nearest;
        desc.mag_filter = FilterMode::Nearest;
    } else {
        warn!("Could not take missing_tex Image sampler_descriptor");
    }
    missing_tex.image = images.add(image);
    missing_tex.material = materials.add(StandardMaterial {
        base_color_texture: Some(missing_tex.image.clone()),
        unlit: true,
        ..default()
    });
}