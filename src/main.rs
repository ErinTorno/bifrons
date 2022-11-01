#![feature(default_free_fn)]
#![feature(float_next_up_down)]
#![feature(hash_drain_filter)]
#![feature(hash_raw_entry)]

use std::{path::Path};

use bevy::asset::{FileAssetIo};
use bevy_inspector_egui::{WorldInspectorPlugin};
use bevy_kira_audio::prelude::*;
use bevy::prelude::*;
use bevy_mod_scripting::prelude::*;
use data::{actor::{Actor, ActorLoader}, level::{Level, LevelLoader}};

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
        .add_plugin(system::action::ActionPlugin)
        .add_plugin(system::camera::CameraPlugin)
        .add_plugin(system::level::LevelPlugin)
        .add_plugin(system::player::PlayerPlugin)
        .add_plugin(system::texture::TexturePlugin)
        .insert_resource(FileAssetIo::new(Path::new("./assets"), false))
        .add_script_host::<LuaScriptHost<()>,_>(CoreStage::PostUpdate)
        .add_asset::<Actor>()
        .add_asset::<Level>()
        .init_asset_loader::<ActorLoader>()
        .init_asset_loader::<LevelLoader>()
        .run();
}