#![feature(default_free_fn)]
#![feature(float_next_up_down)]
#![feature(hash_drain_filter)]
#![feature(hash_raw_entry)]
#![feature(iter_intersperse)]

use std::{path::Path};

use bevy::asset::{FileAssetIo};
use bevy_inspector_egui::{WorldInspectorPlugin};
use bevy_kira_audio::prelude::*;
use bevy::prelude::*;
use bevy_mod_scripting::prelude::*;
use bevy_rapier3d::{prelude::{RapierPhysicsPlugin, NoUserData}, render::RapierDebugRenderPlugin};
use data::{prefab::{Prefab, PrefabLoader}, level::{Level, LevelLoader, LevelPiece, LevelPieceLoader}, formlist::{FormList, FormListLoader}};
use scripting::ManyScriptVars;

mod data;
mod scripting;
mod system;
mod util;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(AudioPlugin)
        .add_plugin(ScriptingPlugin)
        .add_plugin(scripting::ScriptPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(system::action::ActionPlugin)
        .add_plugin(system::anim::AnimPlugin)
        .add_plugin(system::camera::CameraPlugin)
        .add_plugin(system::level::LevelPlugin)
        .add_plugin(system::palette::PalettePlugin)
        .add_plugin(system::prefab::PrefabPlugin)
        .add_plugin(system::scene::ScenePlugin)
        .add_plugin(system::texture::TexturePlugin)
        .insert_resource(FileAssetIo::new(Path::new("./assets"), false))
        .add_script_host::<LuaScriptHost<ManyScriptVars>,_>(CoreStage::PostUpdate)
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