use std::time::Duration;
use std::{collections::{HashMap, HashSet}};

use bevy::asset::LoadState;
use bevy::ecs::system::SystemState;
use bevy::{prelude::*};
use bevy_inspector_egui::InspectorOptions;
use iyes_loopless::prelude::FixedTimestepStage;
use mlua::prelude::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::data::lua::{LuaScript, LuaScriptLoader, InstanceKind, InstanceRef, Hook, LuaWorld, Recipient};
use crate::scripting::event::{constants, ON_UPDATE, ON_INIT};
use crate::scripting::register_lua_mods;
use crate::scripting::registry::{Registry, AssetEventKey};
use crate::scripting::time::LuaTime;
use crate::util::collections::Singleton;

#[derive(Clone, Debug, Default)]
pub struct LuaPlugin;

impl Plugin for LuaPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut on_update = SystemStage::single_threaded();
        on_update
            .add_system(send_on_update)
            .add_system(on_asset_load);

        let mut on_queue_collect = SystemStage::parallel();
        on_queue_collect.add_system(update_script_queue);
        
        app
            .init_resource::<LuaTime>()
            .init_resource::<Registry>()
            .init_resource::<SharedInstances>()
            .add_asset::<LuaScript>()
            .init_asset_loader::<LuaScriptLoader>()
            .add_system(init_lua_script)
            .add_stage_before(
                CoreStage::Update,
                "on_update",
                FixedTimestepStage::new(Duration::from_secs_f32(constants::ON_UPDATE_DELAY), ON_UPDATE).with_stage(on_update),
            )
            .add_stage_after(
                CoreStage::PostUpdate,
                "on_queue_collect",
                on_queue_collect,
            )
            .register_type::<HookCall>()
            .register_type::<InstanceKind>()
            .register_type::<LuaScript>()
            .register_type::<LuaQueue>()
        ;
    }
}

#[derive(Clone, Debug, FromReflect, Reflect)]
pub struct HookCall {
    #[reflect(ignore)]
    pub script_ids: HashSet<u32>,
    pub hook:       Hook,
}
impl HookCall {
    pub fn next_frame(hook: Hook) -> Self { HookCall { script_ids: HashSet::singleton(SharedInstances::COLLECTIVIST_ID), hook }}
}

#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct LuaQueue {
    pub calls: Vec<HookCall>,
}

pub struct LuaInstance {
    pub handle:    Handle<LuaScript>,
    pub path:      String,
    pub result:    Result<InstanceRef, LuaError>,
}

#[derive(Clone, Component, Debug, Default, Deserialize, InspectorOptions, PartialEq, Serialize)]
pub struct ScriptRefs {
    pub ids: HashSet<u32>,

}
impl LuaUserData for ScriptRefs {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("#scriptref{{id = {:?}}}", this.ids)));
    }
}

#[derive(Resource)]
pub struct SharedInstances {
    pub next_id:      u32,
    pub collectivist: InstanceRef,
    pub by_path:      HashMap<String, HashMap<Entity, u32>>,
    pub instances:    HashMap<u32, LuaInstance>,
    pub shared:       HashMap<Handle<LuaScript>, u32>,
    pub updateables:  HashSet<u32>,
}
impl SharedInstances {
    pub const COLLECTIVIST_ID: u32 = 0;
    
    pub fn gen_next_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id - 1
    }
}
impl SharedInstances {
    pub fn foreach<F>(rec: &Recipient, f: F) where F: Fn(&Lua) -> Result<(), LuaError> {
        match rec {
            Recipient::Entity(e) => {

            },
            Recipient::Everyone => {

            },
            Recipient::NoOne => (),
            Recipient::Script(name) => (),
        }
    }
}
impl Default for SharedInstances {
    fn default() -> Self {
        let collectivist = Lua::new();
        register_lua_mods(&collectivist).unwrap();
        collectivist.globals().set("script_id", SharedInstances::COLLECTIVIST_ID).unwrap();
        // todo register world
        let collectivist = RwLock::new(collectivist).into();
        SharedInstances {
            next_id: 1,
            collectivist,
            shared: HashMap::new(),
            instances: HashMap::new(),
            by_path: HashMap::new(),
            updateables: HashSet::new(),
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct ToInitScripts {
    pub handles: HashMap<u32, Handle<LuaScript>>,
}

pub fn send_on_update(
    time:         Res<Time>,
    shared_instances: Res<SharedInstances>,
    mut lua_time: ResMut<LuaTime>,
    mut query:    Query<(&mut LuaQueue, &ScriptRefs)>,
) {
    let elapsed = time.elapsed_seconds_f64();
    let delta = if lua_time.elapsed > 0. { elapsed - lua_time.elapsed } else { 0. };
    lua_time.elapsed = elapsed;
    lua_time.delta = delta;
    let hook = Hook { name: ON_UPDATE.into(), args: (lua_time.clone(),).into() };

    for (mut queue, refs) in query.iter_mut() {
        if refs.ids.intersection(&shared_instances.updateables).next().is_some() {
            queue.calls.push(HookCall::next_frame(hook.clone()));
        }
    }
}

pub fn load_script(script: &LuaScript, world: LuaWorld, id: u32) -> Result<InstanceRef, LuaError> {
    let lua = Lua::new();
    lua.globals().set("world", world)?;
    lua.load(&script.source).exec()?;
    register_lua_mods(&lua)?;
    lua.globals().set("script_id", id)?;
    Ok(RwLock::new(lua).into())
}

pub fn init_lua_script(
    world: &mut World,
    state: &mut SystemState<(
        Commands,
        Res<AssetServer>,
        ResMut<SharedInstances>,
        Res<Assets<LuaScript>>,
        Query<(Entity, &mut ToInitScripts, Option<&mut ScriptRefs>, Option<&mut LuaQueue>)>,
    )>,
) {
    let lua_world = unsafe { LuaWorld::new(world) };
    let (mut commands, asset_server, mut instances, lua_scripts, mut query) = state.get_mut(world);
    'query: for (entity, to_init, script_refs, lua_queue) in query.iter_mut() {
        let mut scripts = HashMap::new();
        for (id, handle) in to_init.handles.iter() {
            if let Some(script) = lua_scripts.get(handle) {
                scripts.insert(handle.clone_weak(), (*id, script));
            } else {
                continue 'query;
            }
        }
        let mut ids = HashSet::new();
        for (handle, (id, script)) in scripts.iter() { 
            let id = *id;
            let get_path = |asset_server: &AssetServer, handle: &Handle<LuaScript>| asset_server.get_handle_path(handle)
                .and_then(|p| p.path().to_str()
                .map(|s| s.to_string()))
                .unwrap_or(format!("pathless/{}", id));
            match script.instance {
                InstanceKind::Unique => {
                    let path = get_path(&asset_server, handle);
                    let result = load_script(&script, lua_world.clone(), id);
                    if let Err(err) = &result {
                        error!("Failed to load {}: {}", path, err);
                    }
                    ids.insert(id);
                    instances.by_path.entry(path.clone())
                        .or_insert_with(|| HashMap::new())
                        .insert(entity, id);
                    instances.instances.insert(id, LuaInstance {
                        handle: handle.clone_weak(),
                        path,
                        result,
                    });
                },
                InstanceKind::Shared => {
                    match instances.shared.get(&handle) {
                        Some(instance_id) => {
                            ids.insert(*instance_id);
                        },
                        None => {
                            let path = get_path(&asset_server, handle);
                            let result = load_script(&script, lua_world.clone(), id);
                            if let Err(err) = &result {
                                error!("Failed to load {}: {}", path, err);
                            }
                            ids.insert(id);
                            instances.by_path.entry(path.clone())
                                .or_insert_with(|| HashMap::new())
                                .insert(entity, id);
                            instances.instances.insert(id, LuaInstance {
                                handle: handle.clone_weak(),
                                path,
                                result,
                            });
                            instances.shared.insert(handle.clone_weak(), id);
                        },
                    }
                },
                InstanceKind::Collectivist => {
                    ids.insert(SharedInstances::COLLECTIVIST_ID);
                    let path = get_path(&asset_server, handle);
                    {
                        let w = instances.collectivist.lock.write();
                        let _ = w.load(&script.source).exec().map_err(|err| {
                            error!("Failed to load {}: {}", path, err);
                        });
                    }
                    instances.by_path.entry(path.clone())
                        .or_insert_with(|| HashMap::new())
                        .insert(entity, id);
                },
            }
        }
        commands.entity(entity)
            .remove::<ToInitScripts>();
        if let Some(mut script_refs) = script_refs {
            script_refs.ids.extend(ids);
        } else {
            commands.entity(entity).insert(ScriptRefs { ids });
        }
        if let Some(mut lua_queue) = lua_queue {
            lua_queue.calls.push(HookCall::next_frame(Hook { name: ON_INIT.to_string(), args: default() }));
        } else {
            commands.entity(entity).insert(LuaQueue { calls: vec![HookCall::next_frame(Hook { name: ON_INIT.to_string(), args: default() })] });
        }
    }
    state.apply(world);
}

pub fn update_script_queue(
    mut si:    ResMut<SharedInstances>,
    mut query: Query<(Entity, &mut LuaQueue, &ScriptRefs), >,
) {
    for (entity, mut queue, script_ref) in query.iter_mut() {
        queue.calls.retain(|HookCall { hook, script_ids }| {
            if script_ids.is_empty() || script_ids.iter().all(|i| *i == SharedInstances::COLLECTIVIST_ID || si.instances.contains_key(i)) {
                for id in script_ref.ids.iter() {
                    let is_updateable = {
                        let lua_inst = si.instances.get(id).unwrap();
                        if let Ok(inst_ref) = &lua_inst.result {
                            let _ = hook.exec(&inst_ref.lock, entity.into()).map_err(|e| {
                                hook.log_err(e);
                            });
                            if *id != SharedInstances::COLLECTIVIST_ID && hook.name.as_str() == ON_INIT {
                                let r = inst_ref.lock.read();
                                let res = r.globals().contains_key(ON_UPDATE);
                                res.unwrap_or(false)
                            } else { false }
                        } else { false }
                    };
                    if is_updateable {
                        si.updateables.insert(*id);
                    }
                }
                false
            } else { true }
        });
    }
}

pub fn on_asset_load(
    mut registry: ResMut<Registry>,
    si:           Res<SharedInstances>,
    asset_server: Res<AssetServer>,
) {
    for (AssetEventKey { entity, handle, script_id }, reg_key) in registry.on_asset_load.drain_filter(|key, _|
        match asset_server.get_load_state(&key.handle.handle) {
            LoadState::Loaded => true,
            LoadState::Failed => {
                let path = asset_server.get_handle_path(&key.handle.handle);
                info!("Asset {:?} failed to load, so all on_load events for {:?} will be dropped", path, key.entity);
                true
            },
            _ => false,
        }
    ) {
        let lua_inst = si.instances.get(&script_id).unwrap();
        if let Ok(inst_ref) = &lua_inst.result {
            let lua = inst_ref.lock.write();
            let v: Vec<LuaFunction> = lua.registry_value(&reg_key).unwrap();
            lua.remove_registry_value(reg_key).unwrap();
            for f in v {
                f.call::<_, ()>(handle.clone()).unwrap();
            }
        } else {
            info!("Lua script {:?} failed to load, so all on_load events for {:?} will be dropped", script_id, entity);
        }
    }
}