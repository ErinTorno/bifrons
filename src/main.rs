#![feature(default_free_fn)]
#![feature(float_next_up_down)]
#![feature(hash_drain_filter)]
#![feature(hash_raw_entry)]
#![feature(iter_intersperse)]
#![feature(let_chains)]

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_kira_audio::prelude::*;
use bevy::{prelude::*, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}};
use bevy_rapier3d::{prelude::{NoUserData, RapierPhysicsPlugin}, render::RapierDebugRenderPlugin};
use data::{prefab::{Prefab, PrefabLoader}, level::{Level, LevelLoader, LevelPiece, LevelPieceLoader}, formlist::{FormList, FormListLoader}, font::{FontLoader, Font}};

mod data;
mod scripting;
mod system;
mod util;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // debug
        .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldInspectorPlugin::default())
        // misc
        .add_plugin(AudioPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default()) if enabled, must disable HDR/bloom
        .add_plugin(system::action::ActionPlugin)
        .add_plugin(system::anim::AnimPlugin)
        .add_plugin(system::camera::CameraPlugin)
        .add_plugin(system::lua::LuaPlugin)
        .add_plugin(system::level::LevelPlugin)
        .add_plugin(system::palette::PalettePlugin)
        .add_plugin(system::prefab::PrefabPlugin)
        .add_plugin(system::scene::ScenePlugin)
        .add_plugin(system::texture::TexturePlugin)
        .add_plugin(system::ui::ScriptingUiPlugin)
        .add_asset::<Font>()
        .add_asset::<FormList>()
        .add_asset::<Prefab>()
        .add_asset::<Level>()
        .add_asset::<LevelPiece>()
        .add_asset::<scripting::ui::elem::Container>()
        .init_asset_loader::<FontLoader>()
        .init_asset_loader::<FormListLoader>()
        .init_asset_loader::<PrefabLoader>()
        .init_asset_loader::<LevelLoader>()
        .init_asset_loader::<LevelPieceLoader>()
        .run();
}