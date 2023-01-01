use bevy::{prelude::*, render::{texture::{ImageSampler}, render_resource::{SamplerDescriptor, FilterMode, Extent3d, TextureDimension, TextureFormat}}, asset::LoadState};
use bevy_egui::EguiContext;

use crate::data::{material::{TexMatInfo, MaterialColors, MaterialsToInit, TextureMaterial, LoadedMat}};

use super::ui::UIAssets;

#[derive(Clone, Debug, Default)]
pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .init_resource::<Background>()
            .init_resource::<ImagesToCheck>()
            .init_resource::<MaterialColors>()
            .init_resource::<MaterialsToInit>()
            .init_resource::<MissingTexture>()
            .init_resource::<TexMatInfo>()
            .add_startup_system(setup_default_textures)
            .add_system(update_image_descriptor)
            .add_system(log_asset_errors)
        ;
    }
}

#[derive(Clone, Debug, Default, Resource)]
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
    tex_mat_info:  Res<TexMatInfo>,
    mut egui_ctx:  ResMut<EguiContext>,
    mut to_check:  ResMut<ImagesToCheck>,
    mut ui_assets: ResMut<UIAssets>,
    mut images:    ResMut<Assets<Image>>,
    mut events:    EventReader<AssetEvent<Image>>,
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
                    let _ = ui_assets.texture_id_or_insert(handle.clone_weak(), egui_ctx.as_mut());
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
                    let _ = ui_assets.texture_id_or_insert(handle.clone_weak(), egui_ctx.as_mut());
                }
            },
            _ => (),
        }
    }
}


#[derive(Clone, Debug, Default, Resource)]
pub struct Background {
    pub color:    Color,
    pub material: Handle<StandardMaterial>,
}

#[derive(Clone, Debug, Default, Resource)]
pub struct MissingTexture {
    pub image:    Handle<Image>,
    pub material: Handle<StandardMaterial>,
}

pub fn setup_default_textures(
    // asset_server:     Res<AssetServer>,
    mut background:   ResMut<Background>,
    mut missing_tex:  ResMut<MissingTexture>,
    mut images:       ResMut<Assets<Image>>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
    mut mat_colors:   ResMut<MaterialColors>,
    mut mats_to_init: ResMut<MaterialsToInit>,
) {
    background.material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..default()
    });
    background.color = Color::BLACK;
    mat_colors.by_handle.insert(background.material.clone_weak(), LoadedMat {
        handle: background.material.clone_weak(),
        tex_mat: TextureMaterial::BACKGROUND.clone(),
    });
    mats_to_init.0.insert(background.material.clone_weak());

    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255, 198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    let image = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    );
    missing_tex.image = images.add(image);
    // missing_tex.image = asset_server.load(resolve_texture_path("missing.png", asset_server.as_ref()));
    missing_tex.material = materials.add(StandardMaterial {
        base_color_texture: Some(missing_tex.image.clone()),
        unlit: true,
        ..default()
    });
}