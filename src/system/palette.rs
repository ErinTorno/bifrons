
use bevy::{prelude::*};

use crate::data::{palette::*};

use super::common::ToInit;

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
            .add_system(init_sync_dyncolors)
        ;
    }
}

fn setup_palette(
    asset_server:        Res<AssetServer>,
    mut current_palette: ResMut<CurrentPalette>,
) {
    current_palette.0 = asset_server.load("palettes/default.palette.ron");
}

fn init_sync_dyncolors(
    mut commands:        Commands,
    mut materials:       ResMut<Assets<StandardMaterial>>,
    mut palettes:        ResMut<Assets<Palette>>,
    mut color_cache:     ResMut<ColorCache>,
    mut current_palette: ResMut<CurrentPalette>,
    query:               Query<(Entity, &Handle<StandardMaterial>, &DynColor), With<ToInit<DynColor>>>,
) {
    let cur_handle = &current_palette.0;
    if let Some(palette) = palettes.get(cur_handle) {
        for (entity, mat_handle, dyn_color) in query.iter() {
            if let Some(material) = materials.get_mut(mat_handle) {
                if let Some(color) = color_cache.color(cur_handle, dyn_color) {
                    material.base_color = color.into();
                    commands.entity(entity)
                        .remove::<ToInit<DynColor>>();
                }
            }
        }
    }
}