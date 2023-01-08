use std::time::Duration;
use std::{collections::{HashMap, HashSet}};

use bevy::asset::LoadState;
use bevy::ecs::system::SystemState;
use bevy::{prelude::*};
use bevy_inspector_egui::{RegisterInspectable};
use indexmap::IndexMap;
use iyes_loopless::prelude::FixedTimestepStage;
use mlua::prelude::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::data::level::{LoadedLevel, LoadedLevelCache};
use crate::data::lua::{LuaScript, LuaScriptLoader, InstanceKind, InstanceRef, Hook, LuaWorld, ScriptVar};
use crate::scripting::bevy_api::LuaEntity;
use crate::scripting::bevy_api::handle::{LuaAssetEventRegistry, AssetEventKey, LuaHandle, AssetKind};
use crate::scripting::event::{constants, ON_UPDATE, ON_INIT, EventFlag, ON_ROOM_REVEAL};
use crate::scripting::register_lua_mods;
use crate::scripting::time::LuaTime;
use crate::scripting::ui::atom::LuaAtomRegistry;

#[derive(Clone, Debug, Default)]
pub struct LuaPlugin;

impl Plugin for LuaPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut on_update = SystemStage::single_threaded();
        on_update
            .add_system(update_script_event_queue)
            .add_system(on_asset_load);

        let mut on_queue_collect = SystemStage::parallel();
        on_queue_collect.add_system(update_script_queue);
        
        app
            .init_resource::<LuaAtomRegistry>()
            .init_resource::<LuaEventQueue>()
            .init_resource::<LuaTime>()
            .init_resource::<LuaAssetEventRegistry>()
            .init_resource::<SharedInstances>()
            .add_asset::<LuaScript>()
            .init_asset_loader::<LuaScriptLoader>()
            .add_system(init_lua_script)
            .add_stage_before(
                CoreStage::Update,
                "lua_events",
                FixedTimestepStage::new(Duration::from_secs_f32(constants::ON_UPDATE_DELAY), ON_UPDATE).with_stage(on_update),
            )
            .add_stage_after(
                CoreStage::PostUpdate,
                "lua_queue_collect",
                on_queue_collect,
            )
            .register_type::<InstanceKind>()
            .register_type::<LuaScript>()
            .register_inspectable::<ScriptVar>()
        ;
    }
}

#[derive(Clone, Debug)]
pub struct HookCall {
    pub script_ids: HashSet<u32>,
    pub hook:       Hook,
}
impl HookCall {
    pub fn next_frame(hook: Hook) -> Self { HookCall { script_ids: HashSet::new(), hook }}
}
#[derive(Clone, Debug)]
pub struct EventCall {
    pub flag: EventFlag,
    pub hook: Hook,
}

#[derive(Clone, Component, Debug, Default)]
pub struct LuaQueue {
    pub calls: Vec<HookCall>,
}
#[derive(Clone, Debug, Default, Resource)]
pub struct LuaEventQueue {
    pub calls: Vec<EventCall>,
}

pub struct LuaInstance {
    pub handle:    Handle<LuaScript>,
    pub path:      String,
    pub result:    Result<InstanceRef, LuaError>,
}

#[derive(Clone, Component, Debug, Default, Deserialize, PartialEq, Serialize)]
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
    pub event_flags:  HashMap<u32, EventFlag>,
}
impl SharedInstances {
    pub const COLLECTIVIST_ID: u32 = 0;
    
    pub fn gen_next_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id - 1
    }

    pub fn has_event_flags(&self, flags: EventFlag, script_id: u32) -> bool {
        self.event_flags.get(&script_id).map(|i| *i).unwrap_or(EventFlag::empty()).contains(flags)
    }
}
impl Default for SharedInstances {
    fn default() -> Self {
        let collectivist = Lua::new();
        collectivist.globals().set("script_id", SharedInstances::COLLECTIVIST_ID).unwrap();
        // todo register world
        let collectivist = RwLock::new(collectivist).into();
        SharedInstances {
            next_id: 1,
            collectivist,
            shared: HashMap::new(),
            instances: HashMap::new(),
            by_path: HashMap::new(),
            event_flags: HashMap::new(),
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct ToInitScripts {
    pub handles: IndexMap<u32, Handle<LuaScript>>,
}

pub fn load_script_on_lua(lua: &Lua, script: &LuaScript, world: LuaWorld, id: u32) -> Result<(), LuaError> {
    lua.globals().set("world", world)?;
    register_lua_mods(&lua)?;
    lua.globals().set("script_id", id)?;
    lua.load(&script.source).exec()?;
    Ok(())
}

pub fn load_script(script: &LuaScript, world: LuaWorld, id: u32) -> Result<InstanceRef, LuaError> {
    let lua = Lua::new();
    load_script_on_lua(&lua, script, world, id)?;
    Ok(RwLock::new(lua).into())
}

pub fn init_lua_script(
    world: &mut World,
    state: &mut SystemState<(
        Commands,
        Res<AssetServer>,
        ResMut<SharedInstances>,
        Res<Assets<LuaScript>>,
        Local<bool>,
        Query<(Entity, &mut ToInitScripts, Option<&mut ScriptRefs>, Option<&mut LuaQueue>)>,
    )>,
) {
    let lua_world = unsafe { LuaWorld::new(world) };
    let (mut commands, asset_server, mut instances, lua_scripts, mut is_collectivist_empty, mut query) = state.get_mut(world);
    'query: for (entity, to_init, script_refs, lua_queue) in query.iter_mut() {
        let mut scripts = IndexMap::new();
        for (id, handle) in to_init.handles.iter() {
            if let Some(script) = lua_scripts.get(handle) {
                scripts.insert(handle.clone_weak(), (*id, script));
            } else {
                continue 'query;
            }
        }
        
        if let Some(mut lua_queue) = lua_queue {
            let mut calls = vec![HookCall::next_frame(Hook { name: ON_INIT.to_string(), args: default() })];
            calls.extend(lua_queue.calls.drain(0..));
            lua_queue.calls = calls;
        } else {
            commands.entity(entity).insert(LuaQueue { calls: vec![HookCall::next_frame(Hook { name: ON_INIT.to_string(), args: default() })] });
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
                    ids.insert(id);
                    instances.by_path.entry(path.clone())
                        .or_insert_with(|| HashMap::new())
                        .insert(entity, id);
                    let result = load_script(&script, lua_world.clone(), id);
                    if let Err(err) = &result {
                        error!("Failed to load {}: {}", path, err);
                    }
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
                            ids.insert(id);
                            instances.by_path.entry(path.clone())
                                .or_insert_with(|| HashMap::new())
                                .insert(entity, id);
                            let result = load_script(&script, lua_world.clone(), id);
                            if let Err(err) = &result {
                                error!("Failed to load {}: {}", path, err);
                            }
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
                    instances.by_path.entry(path.clone())
                        .or_insert_with(|| HashMap::new())
                        .insert(entity, id);
                    {
                        let w = instances.collectivist.lock.write();
                        if *is_collectivist_empty {
                            let _ = load_script_on_lua(&w, script, lua_world.clone(), SharedInstances::COLLECTIVIST_ID).map_err(|err| {
                                error!("Failed to load {}: {}", path, err);
                            });

                            *is_collectivist_empty = false;
                        } else {
                            let _ = w.load(&script.source).exec().map_err(|err| {
                                error!("Failed to load {}: {}", path, err);
                            });
                        }
                    }
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
    }
    state.apply(world);
}

pub fn update_script_queue(
    mut si:    ResMut<SharedInstances>,
    mut query: Query<(Entity, &mut LuaQueue, &ScriptRefs)>,
) {
    for (entity, mut queue, script_ref) in query.iter_mut() {
        if !queue.calls.is_empty() {
            // info!("{:?} trying queue {:?}", entity, queue.calls);
        }
        queue.calls.retain(|HookCall { hook, script_ids }| {
            if script_ids.is_empty() || script_ids.iter().all(|i| *i == SharedInstances::COLLECTIVIST_ID || si.instances.contains_key(i)) {
                // info!("{:?} consuming hook {:?}", entity, hook);
                for id in script_ref.ids.iter() {
                    let lua_inst = si.instances.get(id).unwrap();
                    if let Ok(inst_ref) = &lua_inst.result {
                        let _ = hook.exec(&inst_ref.lock, entity.into()).map_err(|e| {
                            hook.log_err(e);
                        });
                        if *id != SharedInstances::COLLECTIVIST_ID && hook.name.as_str() == ON_INIT {
                            let mut events = EventFlag::empty();
                            if (|| {
                                let r = inst_ref.lock.read();
                                let globals = r.globals();
                                if globals.contains_key(ON_UPDATE)? {
                                    events |= EventFlag::ON_UPDATE;
                                }
                                if globals.contains_key(ON_ROOM_REVEAL)? {
                                    events |= EventFlag::ON_ROOM_REVEAL;
                                }
                                Ok(())
                            })().map_err(|e: mlua::Error| {
                                error!("Failed to get EventFlags for script {}: {}", *id, e);
                            }).is_ok() {
                                si.event_flags.insert(*id, events);
                            }
                        }
                    }
                }
                false
            } else { true }
        });
    }
}

pub fn update_script_event_queue(
    time:                Res<Time>,
    mut lua_event_queue: ResMut<LuaEventQueue>,
    mut lua_time:        ResMut<LuaTime>,
    si:                  ResMut<SharedInstances>,
    query:               Query<(Entity, &ScriptRefs), >,
) {
    let elapsed = time.elapsed_seconds_f64();
    let delta = if lua_time.elapsed > 0. { elapsed - lua_time.elapsed } else { 0. };
    lua_time.elapsed = elapsed;
    lua_time.delta = delta;

    for EventCall { flag, hook } in lua_event_queue.calls.drain(..) {
        for (entity, script_ref) in query.iter() {
            for id in script_ref.ids.iter() {
                if si.has_event_flags(flag, *id) {
                    let lua_inst = si.instances.get(id).unwrap();
                    if let Ok(inst_ref) = &lua_inst.result {
                        let _ = hook.exec(&inst_ref.lock, entity.into()).map_err(|e| {
                            hook.log_err(e);
                        });
                    }
                }
            }
        }
    }
    for (entity, script_ref) in query.iter() {
        for id in script_ref.ids.iter() {
            if si.has_event_flags(EventFlag::ON_UPDATE, *id) {
                let lua_inst = si.instances.get(id).unwrap();
                if let Ok(inst_ref) = &lua_inst.result {
                    let _ = (|| {
                        let lua = inst_ref.lock.write();
                        lua.globals().set("entity", LuaEntity(entity))?;
                        if let Some(f) = lua.globals().get::<_, Option<LuaFunction>>(ON_UPDATE)? {
                            f.call(lua_time.clone().to_lua_multi(&lua)?)?;
                        }
                        Ok(())
                    })().map_err(|e: mlua::Error| {
                        error!("{:?} script id #{:?} on_update error {}", entity, id, e);
                    });
                }
            }
        }
    }
}

pub fn on_asset_load(
    loaded_levels: Res<Assets<LoadedLevel>>,
    ll_cache:      Res<LoadedLevelCache>,
    si:            Res<SharedInstances>,
    asset_server:  Res<AssetServer>,
    mut registry:  ResMut<LuaAssetEventRegistry>,
) {
    for (AssetEventKey { entity, handle, script_id }, reg_key) in registry.on_asset_load.drain_filter(|key, _|
        match {
            match &key.handle {
                LuaHandle { kind: AssetKind::Level { is_loaded }, handle } => if {
                    if *is_loaded {
                        loaded_levels.contains(&handle.clone_weak().typed())
                    } else {
                        ll_cache.loaded_by_level.contains_key(&handle.clone_weak().typed())
                    }
                } { LoadState::Loaded } else { LoadState::Loading },
                _ => asset_server.get_load_state(&key.handle.handle),
            }
         } {
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