#![feature(default_free_fn)]
#![feature(float_next_up_down)]
#![feature(hash_drain_filter)]
#![feature(hash_raw_entry)]
#![feature(iter_intersperse)]

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::prelude::*;
use bevy::{prelude::*, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}};
use bevy_rapier3d::{prelude::{NoUserData, RapierPhysicsPlugin}, render::RapierDebugRenderPlugin};
use data::{prefab::{Prefab, PrefabLoader}, level::{Level, LevelLoader, LevelPiece, LevelPieceLoader}, formlist::{FormList, FormListLoader}};

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
        .add_plugin(WorldInspectorPlugin)
        // misc
        .add_plugin(AudioPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(system::action::ActionPlugin)
        .add_plugin(system::anim::AnimPlugin)
        .add_plugin(system::camera::CameraPlugin)
        .add_plugin(system::lua::LuaPlugin)
        .add_plugin(system::level::LevelPlugin)
        .add_plugin(system::palette::PalettePlugin)
        .add_plugin(system::prefab::PrefabPlugin)
        .add_plugin(system::scene::ScenePlugin)
        .add_plugin(system::texture::TexturePlugin)
        .add_asset::<FormList>()
        .add_asset::<Prefab>()
        .add_asset::<Level>()
        .add_asset::<LevelPiece>()
        .init_asset_loader::<FormListLoader>()
        .init_asset_loader::<PrefabLoader>()
        .init_asset_loader::<LevelLoader>()
        .init_asset_loader::<LevelPieceLoader>()
        .run();
}