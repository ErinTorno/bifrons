
use std::collections::HashMap;

use bevy::{prelude::*};
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::{data::{palette::*, material::{MaterialColors, MaterialsToInit}}, scripting::color::RgbaColor};

use super::{lua::{SharedInstances, ToInitScripts}};

#[derive(Clone, Debug, Default)]
pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .init_resource::<ColorCache>()
            .add_asset::<Palette>()
            .init_asset_loader::<PaletteLoader>()
            .add_startup_system(load_default_palette)
            .add_system(initialize_palettes)
            .add_system(setup_palette
                .run_if_resource_exists::<LoadingPalette>())
            .add_system(init_sync_dyncolors)
            .add_system(sync_light_dyncolors)
            .register_type::<DynColor>()
            .register_type::<RgbaColor>()
            .register_type::<LoadingPalette>()
            .register_type::<SingleColored>()
        ;
    }
}


#[derive(Clone, Debug, Reflect, Resource)]
pub struct LoadingPalette {
    pub handle: Handle<Palette>,
}

fn load_default_palette(
    mut commands:        Commands,
    asset_server:        Res<AssetServer>,
    material_colors:     Res<MaterialColors>,
    mut mats_to_init:    ResMut<MaterialsToInit>,
) {
    let handle = asset_server.load("palettes/default.palette.ron");
    let entity = commands.spawn(Name::new("palette_entity")).id();
    commands.insert_resource(LoadedPalettes {
        current_handle: handle.clone(),
        current_state:  LoadedPaletteState { entity, script_id: SharedInstances::COLLECTIVIST_ID },
        by_handle:      HashMap::new(),
    });
    mats_to_init.0.extend(material_colors.by_handle.keys().map(|h| h.clone_weak()));
}

fn setup_palette(
    mut commands:        Commands,
    asset_server:        Res<AssetServer>,
    palettes:            Res<Assets<Palette>>,
    loading_palette:     Res<LoadingPalette>,
    material_colors:     Res<MaterialColors>,
    // mut ambient:         ResMut<AmbientLight>,
    mut lua_instances:   ResMut<SharedInstances>,
    mut mats_to_init:    ResMut<MaterialsToInit>,
    mut loaded_palettes: ResMut<LoadedPalettes>,
    query_to_init:       Query<&ToInitScripts>,
    mut palette_entity:  Local<Option<Entity>>,
) {
    fn finish_loading(commands: &mut Commands, palette_entity: &mut Option<Entity>) {
        *palette_entity = None;
        commands.remove_resource::<LoadingPalette>();
    }

    if let Some(st) = loaded_palettes.by_handle.get(&loading_palette.handle).cloned() {
        loaded_palettes.current_handle = loading_palette.handle.clone();
        loaded_palettes.current_state = st;
        mats_to_init.0.extend(material_colors.by_handle.keys().map(|h| h.clone_weak()));
        finish_loading(&mut commands, &mut palette_entity);
    } else if let Some(palette) = palettes.get(&loading_palette.handle) {
        let entity = palette_entity.unwrap_or_else(|| {
            loaded_palettes.by_handle.get(&loading_palette.handle).map(|st| st.entity).unwrap_or_else(|| {
                commands.spawn_empty().id()
            })
        });
        *palette_entity = Some(entity);
        if let Some(file) = palette.get_script() {
            if let Some(hs) = lua_instances.by_path.get(file) && let Some(script_id) = hs.get(&entity) {
                info!("Script refs loaded, time to switch");
                let st = LoadedPaletteState { entity, script_id: *script_id };
                loaded_palettes.current_handle = loading_palette.handle.clone();
                loaded_palettes.current_state = st;
                loaded_palettes.by_handle.insert(loading_palette.handle.clone(), st);
                mats_to_init.0.extend(material_colors.by_handle.keys().map(|h| h.clone_weak()));
                finish_loading(&mut commands, &mut palette_entity);
            } else if let Err(_) = query_to_init.get(entity) {
                let mut handles = HashMap::new();
                handles.insert(lua_instances.gen_next_id(), asset_server.load(file));
                commands.entity(entity)
                    .insert(ToInitScripts { handles });
            }
        } else {
            let st = LoadedPaletteState {
                entity,
                script_id: SharedInstances::COLLECTIVIST_ID,
            };
            loaded_palettes.current_handle = loading_palette.handle.clone();
            loaded_palettes.current_state = st;
            loaded_palettes.by_handle.insert(loading_palette.handle.clone(), st);
            mats_to_init.0.extend(material_colors.by_handle.keys().map(|h| h.clone_weak()));
            finish_loading(&mut commands, &mut palette_entity);
        }
    }
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
    loaded_palettes:  Res<LoadedPalettes>,
    shared_instances: Res<SharedInstances>,
    mut directional_lights: Query<(&SingleColored, &mut DirectionalLight), (Without<PointLight>,       Without<SpotLight>)>,
    mut point_lights:       Query<(&SingleColored, &mut PointLight),       (Without<DirectionalLight>, Without<SpotLight>)>,
    mut spot_lights:        Query<(&SingleColored, &mut SpotLight),        (Without<DirectionalLight>, Without<PointLight>)>,
) {
    if let Some(palette) = palettes.get(&loaded_palettes.current_handle) {
        if let Some(i) = loaded_palettes.lua_instance(shared_instances.as_ref()) {
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
    loaded_palettes:  Res<LoadedPalettes>,
    material_colors:  Res<MaterialColors>,
    shared_instances: Res<SharedInstances>,
    mut mats_to_init: ResMut<MaterialsToInit>,
) {
    if let Some(palette) = palettes.get(&loaded_palettes.current_handle) {
        mats_to_init.0.retain(|handle| {
            if let Some(material) = materials.get_mut(handle) {
                let loaded_mat = &material_colors.by_handle[handle];
                let tex = &loaded_mat.tex_mat;
                if let Some(i) = loaded_palettes.lua_instance(shared_instances.as_ref()) {
                    clear_color.0       = color_cache.rgba(&palette.background, palette, i).into();
                    material.base_color = color_cache.rgba(&tex.color,          palette, i).into();
                    material.emissive   = color_cache.rgba(&tex.emissive_color, palette, i).into();
                }
                false
            } else { true }
        });
    }
}