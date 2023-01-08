use std::collections::{HashMap};

use bevy::{prelude::*, utils::BoxedFuture, asset::*, reflect::TypeUuid};
use indexmap::IndexMap;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{util::{ron_options, easy_hash}, scripting::{LuaMod, bevy_api::{handle::LuaHandle, LuaEntity, math::LuaVec3}}, system::{common::{fix_missing_extension, ToInitHandle}, lua::SharedInstances}};

use super::{geometry::{Geometry, Light}, material::TextureMaterial, stat::Attributes, lua::{LuaScriptVars, LuaWorld, LuaScript, TransVar}};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PrefabLocation {
    Free(Vec3),
    // Cell(CellID),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrefabInstance {
    #[serde(default)]
    pub label: Option<String>,
    pub asset: String,
    pub at:    PrefabLocation,
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(default = "default_room_child")]
    pub room_child: bool,
    #[serde(default)]
    pub script_vars: LuaScriptVars,
    #[serde(default)]
    pub attributes: Option<Attributes>,
}
fn default_room_child() -> bool { true }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PointEntity {
    pub pos:  Vec3,
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Room {
    #[serde(default)]
    pub reveal_before_entry: bool,
    #[serde(default)]
    pub pos:            Vec3,
    #[serde(default)]
    pub prefabs:        Vec<PrefabInstance>,
    #[serde(default)]
    pub geometry:       Vec<Geometry>,
    #[serde(default)]
    pub lights:         Vec<Light>,
    #[serde(default)]
    pub point_entities: Vec<PointEntity>,
}

#[derive(Clone, Component, Debug, Default, Deserialize, Serialize)]
pub struct InRoom {
    pub room: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, TypeUuid)]
#[uuid = "a491e648-a317-40e9-a1eb-69f4532f2258"]
pub struct Level {
    #[serde(default = "default_scripts")]
    pub scripts:     Vec<String>,
    #[serde(default)]
    pub script_vars: LuaScriptVars,
    pub materials:   HashMap<String, TextureMaterial>,
    pub rooms:       HashMap<String, Room>,
}
pub fn default_scripts() -> Vec<String> { Vec::new() }

#[derive(Default)]
pub struct LevelLoader;

impl AssetLoader for LevelLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let level: Level = ron_options().from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(level));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["level.ron"]
    }
}

#[derive(Default, Resource)]
pub struct LoadedLevelCache {
    pub loaded_by_level: HashMap<Handle<Level>, Handle<LoadedLevel>>,
}

// pub enum Loadedness {
//     Minimal,
//     Graphics {
//         pub materials: HashMap<String, Handle<StandardMaterial>>,
//     },
// }

#[derive(Clone, Component, Debug, TypeUuid)]
#[uuid = "10ad3d13-b311-4ab7-a40c-05663c428eb8"]
pub struct LoadedLevel {
    pub level_handle: Option<Handle<Level>>,
    pub this_handle:  Handle<LoadedLevel>,
    pub scripts:      IndexMap<u32, Handle<LuaScript>>,
    pub script_vars:  HashMap<String, TransVar>,
    pub materials:    HashMap<String, TextureMaterial>,
    pub rooms:        HashMap<String, Room>,
}
impl LuaUserData for LoadedLevel {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_material", |_, this, k: String| Ok(this.materials.get(&k).cloned()));
        methods.add_method_mut("remove_material", |_, this, k: String| Ok(this.materials.remove(&k)));
        methods.add_method_mut("set_material", |_, this, (k, v): (String, TextureMaterial)| Ok(this.materials.insert(k, v)));

        methods.add_method_mut("add_script", |lua, this, h: LuaHandle| {
            let handle = h.try_script()?;
            let world  = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w  = world.write();
            let mut si = w.resource_mut::<SharedInstances>();
            this.scripts.insert(si.gen_next_id(), handle);
            Ok(()) 
        });
        methods.add_method_mut("remove_script", |_, this, h: LuaHandle| {
            let handle = h.try_script()?;
            let len    = this.scripts.len();
            this.scripts.retain(|_, that_handle| that_handle == &handle);
            Ok(len - this.scripts.len()) 
        });

        methods.add_method("get_var", |_, this, k: String| Ok(this.script_vars.get(&k).cloned()));
        methods.add_method_mut("set_var", |_, this, (k, v): (String, TransVar)| Ok(this.script_vars.insert(k, v)));

        methods.add_method("is_defined_in_file", |_, this, ()| Ok(this.level_handle.is_some()));
    
        methods.add_method("spawn", |lua, this, table: Option<LuaTable>| {
            let table        = if let Some(t) = table { t } else { lua.create_table()? };
            let translation  = table.get::<_, Option<LuaVec3>>("position")?.map(|p| p.0).unwrap_or(Vec3::ZERO);
            let rotation     = table.get::<_, Option<LuaVec3>>("rotation")?.map(|p| p.0).unwrap_or(Vec3::ZERO);
            let is_revealed  = table.get::<_, Option<bool>>("is_revealed")?.unwrap_or(true);
            let debug_name   = table.get::<_, Option<String>>("debug_name")?;

            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let id = {
                let name = {
                    let w = world.read();
                    let asset_server = w.resource::<AssetServer>();
                    Name::from(debug_name.unwrap_or_else(|| match this.level_handle.as_ref().and_then(|h| asset_server.get_handle_path(h)) {
                        Some(p) => p.path().to_string_lossy().to_string(),
                        None    => format!("unsaved_lvl#{}", easy_hash(&this.this_handle)),
                    }))
                };
                let mut w = world.write();
                w.spawn((
                    name,
                    ToInitHandle(this.this_handle.clone_weak()),
                    TransformBundle {
                        local: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z)).with_translation(translation),
                        ..default()
                    },
                    VisibilityBundle {
                        visibility: Visibility { is_visible: is_revealed },
                        ..VisibilityBundle::default()
                    },
                )).id()
            };

            if let Some(parent) = table.get::<_, Option<LuaEntity>>("parent")? {
                let mut w = world.write();
                if let Some(mut parent_entity) = w.get_entity_mut(parent.0) {
                    parent_entity.push_children(&[id]);
                }
            }
            Ok(LuaEntity::new(id))
        });
    }
}
impl LuaMod for LoadedLevel {
    fn mod_name() -> &'static str { "Level" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("add", lua.create_function(|lua, loaded_level: LoadedLevel| {
            let world   = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w   = world.write();
            let mut lls = w.resource_mut::<Assets<LoadedLevel>>();
            Ok(LuaHandle::from(lls.add(loaded_level)))
        })?)?;
        table.set("load", lua.create_function(|lua, path: String| {
            let path = fix_missing_extension::<LevelLoader>(path);
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let asset_server = w.resource::<AssetServer>();
            let handle: Handle<Level> = asset_server.load(&path);
            Ok(LuaHandle::from(handle))
        })?)?;
        Ok(())
    }
}