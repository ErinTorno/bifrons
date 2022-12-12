
use bevy::{prelude::*};

use crate::{data::{palette::*, material::{MaterialColors, MaterialsToInit}}, scripting::color::RgbaColor};

use super::{lua::SharedInstances};

#[derive(Clone, Debug, Default)]
pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .init_resource::<ColorCache>()
            .init_resource::<CurrentPalette>()
            .add_asset::<Palette>()
            .init_asset_loader::<PaletteLoader>()
            .add_startup_system(setup_palette)
            .add_system(initialize_palettes)
            .add_system(init_sync_dyncolors)
            .add_system(sync_light_dyncolors)
            .register_type::<DynColor>()
            .register_type::<RgbaColor>()
            .register_type::<SingleColored>()
        ;
    }
}

fn setup_palette(
    asset_server:        Res<AssetServer>,
    material_colors:     Res<MaterialColors>,
    // mut ambient:         ResMut<AmbientLight>,
    mut mats_to_init:    ResMut<MaterialsToInit>,
    mut current_palette: ResMut<CurrentPalette>,
) {
    current_palette.handle = asset_server.load("palettes/default.palette.ron");
    mats_to_init.0.extend(material_colors.by_handle.keys().map(|h| h.clone_weak()));
    // ambient.brightness = 0.;
}

fn initialize_palettes(
    mut palettes: ResMut<Assets<Palette>>,
    mut events: EventReader<AssetEvent<Palette>>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut palette) = palettes.get_mut(handle) {
                    // gotta change the Handle::default() this has to it's actual Handle
                    palette.handle = handle.clone_weak();
                }
            },
            AssetEvent::Modified { handle } => {
                if let Some(mut palette) = palettes.get_mut(handle) {
                    // same
                    palette.handle = handle.clone_weak();
                }
            },
            _ => (),
        }
    }
}

fn sync_light_dyncolors(
    palettes:         Res<Assets<Palette>>,
    mut color_cache:  ResMut<ColorCache>,
    current_palette:  Res<CurrentPalette>,
    shared_instances: Res<SharedInstances>,
    mut directional_lights: Query<(&SingleColored, &mut DirectionalLight), (Without<PointLight>, Without<SpotLight>)>,
    mut point_lights:       Query<(&SingleColored, &mut PointLight),       (Without<DirectionalLight>, Without<SpotLight>)>,
    mut spot_lights:        Query<(&SingleColored, &mut SpotLight),        (Without<DirectionalLight>, Without<PointLight>)>,
) {
    let cur_handle = &current_palette.handle;
    if let Some(palette) = palettes.get(cur_handle) {
        if let Some(i) = current_palette.lua_instance(shared_instances.as_ref()) {
            for (single_color, mut light) in directional_lights.iter_mut() {
                light.color = color_cache.rgba(&single_color.0, palette, i).into();
            }
            for (single_color, mut light) in point_lights.iter_mut() {
                light.color = color_cache.rgba(&single_color.0, palette, i).into();
            }
            for (single_color, mut light) in spot_lights.iter_mut() {
                light.color = color_cache.rgba(&single_color.0, palette, i).into();
            }
        }
    }
}

fn init_sync_dyncolors(
    mut clear_color:  ResMut<ClearColor>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
    palettes:         Res<Assets<Palette>>,
    mut color_cache:  ResMut<ColorCache>,
    current_palette:  Res<CurrentPalette>,
    material_colors:  Res<MaterialColors>,
    shared_instances: Res<SharedInstances>,
    mut mats_to_init: ResMut<MaterialsToInit>,
) {
    if let Some(palette) = palettes.get(&current_palette.handle) {
        mats_to_init.0.retain(|handle| {
            if let Some(material) = materials.get_mut(handle) {
                let loaded_mat = &material_colors.by_handle[handle];
                let tex = &loaded_mat.tex_mat;
                if let Some(i) = current_palette.lua_instance(shared_instances.as_ref()) {
                    clear_color.0       = color_cache.rgba(&palette.background, palette, i).into();
                    material.base_color = color_cache.rgba(&tex.color,          palette, i).into();
                    material.emissive   = color_cache.rgba(&tex.emissive_color, palette, i).into();
                }
                false
            } else { true }
        });
    }
}