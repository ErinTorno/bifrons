use std::{collections::{HashMap, VecDeque}, fs::File, io::Write, path::PathBuf};
use bevy::{prelude::{*}, asset::{FileAssetIo, LoadState}};
use indexmap::IndexMap;
use iyes_loopless::prelude::IntoConditionalSystem;
use ron::ser::PrettyConfig;

use crate::{data::{module::{Module, ModuleLoader, ModList, ModEntry}, lua::LuaScript, assetio::{VirtualFileOverrides}}, system::lua::ToInitScripts, util::ron_options};

use super::lua::{SharedInstances, LuaQueue};

#[derive(Clone, Debug, Default)]
pub struct ModulePlugin;

impl Plugin for ModulePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_asset::<Module>()
            .add_asset_loader(ModuleLoader::default())
            .init_resource::<LoadedModList>()
            .init_resource::<ModLoadState>()
            .add_startup_system(setup_modlist)
            .add_system(load_mods.run_if_resource_exists::<ModLoadState>())
        ;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct LoadedModList {
    pub modlist:     ModList,
    pub handles:     HashMap<String, Handle<Module>>,
    pub root_dirs:   HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub enum ModLoadState {
    #[default]
    LoadingModDefs,
    LoadingScripts {
        remaining: VecDeque<Vec<String>>,
        current:   Vec<Entity>,
    },
}

pub const THIS_MODLIST_FILE: &str = "this.modlist.ron";

pub fn setup_modlist(
    asset_server:     Res<AssetServer>,
    mut loaded_ml:    ResMut<LoadedModList>,
) {
    let mut modlist_path = FileAssetIo::get_base_path();
    modlist_path.push(THIS_MODLIST_FILE);

    if modlist_path.is_file() {
        let bytes = match std::fs::read(modlist_path) {
            Ok(file) => file,
            Err(e)   => panic!("Unable to open {}: {}", THIS_MODLIST_FILE, e),
        };

        loaded_ml.modlist = match ron_options().from_bytes(&bytes) {
            Ok(file) => file,
            Err(e)   => panic!("Unable to deserialize {}: {}", THIS_MODLIST_FILE, e),
        };
    } else {
        loaded_ml.modlist.entries = vec![ModEntry {
            file:     "assets/core.mod.ron".to_string(),
            settings: HashMap::new(),
            name:     "core".to_string(),
        }];

        let s = ron::ser::to_string_pretty(&loaded_ml.modlist, PrettyConfig::default())
            .expect("ModList should never fail to serialize");
        let mut file = match File::create(modlist_path) {
            Ok(file) => file,
            Err(e)   => panic!("Unable to create {}: {}", THIS_MODLIST_FILE, e),
        };
        if let Err(e) = file.write(s.as_bytes()) {
            panic!("Unable to write to {}: {}", THIS_MODLIST_FILE, e)
        }
    }
    let handles = loaded_ml.modlist.entries.iter()
        .map(|me| (me.name.clone(), asset_server.load(&format!("../{}", me.file))))
        .collect();
    loaded_ml.handles = handles;

    loaded_ml.root_dirs = loaded_ml.modlist.entries.iter()
        .map(|e| (e.name.clone(), {
            let mut path = PathBuf::from(&e.file);
            path.pop();
            if !path.is_dir() {
                panic!("Failed to get directory of mod {}: found {}", e.name, path.to_string_lossy())
            }
            path.to_string_lossy().to_string()
        }))
        .collect();
}

pub fn load_mods(
    mut commands:       Commands,
    mods:               Res<Assets<Module>>,
    loaded_ml:          Res<LoadedModList>,
    asset_server:       Res<AssetServer>,
    mut file_overrides: ResMut<VirtualFileOverrides>,
    mut lua_instances:  ResMut<SharedInstances>,
    mut mls:            ResMut<ModLoadState>,
    query:              Query<Entity, With<LuaQueue>>,
) {
    match mls.as_mut() {
        ModLoadState::LoadingModDefs => {
            let mut is_done = true;
            for (name, handle) in loaded_ml.handles.iter() {
                match asset_server.get_load_state(handle) {
                    LoadState::Failed => {
                        panic!("{}.mod.ron failed to load", name);
                    },
                    LoadState::Loaded => (),
                    _ => { is_done = false; },
                }
            }

            if is_done {
                let mod_map = loaded_ml.handles.iter()
                    .map(|(k, v)| (k.clone(), mods.get(&v).unwrap()))
                    .collect();
        
                let ordered = match Module::sorted_load_order(&mod_map) {
                    Ok(v) => v,
                    Err(e) => { panic!("Modlist validation failed: {:?}", e) },
                };
                let mod_path_load_order = ordered.iter()
                    .map(|v| v.iter().map(|s| &loaded_ml.root_dirs[*s]).collect())
                    .collect();
                file_overrides.populate_files(&mod_path_load_order);

                let remaining = ordered.into_iter().map(|v| v.into_iter().cloned().collect()).collect();
                *mls = ModLoadState::LoadingScripts { remaining, current: Vec::new() };
            }
        },
        ModLoadState::LoadingScripts { remaining, current } => {
            if remaining.is_empty() {
                commands.remove_resource::<ModLoadState>();
            } else {
                if current.iter().all(|e| query.contains(*e)) {
                    if let Some(wave) = remaining.pop_front() {
                       *current = wave.iter().map(|mod_name| {
                            let handle = loaded_ml.handles.get(mod_name).unwrap();
                            let module = mods.get(handle).unwrap();

                            let handles: IndexMap<u32, Handle<LuaScript>> = module.startup_scripts.iter()
                                .map(|s| (lua_instances.gen_next_id(), asset_server.load(s)))
                                .collect();
                            commands.spawn((
                                LuaQueue::default(),
                                ToInitScripts { handles },
                            )).id()
                        }).collect();
                    } else {
                        commands.remove_resource::<ModLoadState>();
                    }
                }
            }
        },
    } 
}