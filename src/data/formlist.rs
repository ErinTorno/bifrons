use std::collections::{HashMap, HashSet};

use bevy::{prelude::*, asset::{LoadContext, AssetLoader, LoadedAsset}, utils::BoxedFuture, reflect::TypeUuid};
use bevy_mod_scripting::{lua::api::bevy::{LuaWorld, LuaEntity}};
use mlua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{scripting::{LuaScriptVars, LuaMod, LuaHandle, random::random_range}, system::common::fix_missing_extension};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RolledRow {
    pub asset:  String,
    #[serde(default = "default_chance")]
    pub chance: f32,
    #[serde(default)]
    pub vars:   LuaScriptVars,
}
pub fn default_chance() -> f32 { 1. }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WeightedRow {
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub vars:   LuaScriptVars,
}
pub fn default_weight() -> f32 { 1. }

#[derive(Clone, Debug, Deserialize, Serialize, TypeUuid)]
#[uuid = "f139a396-a00e-47cc-a849-ae2fc3ecbf26"]
pub enum FormList {
    RollEach(Vec<RolledRow>),
    Union   (HashSet<String>),
    Weighted(HashMap<String, WeightedRow>),
}
impl FormList {
    pub fn inject_roll_each(&mut self, commands: &Vec<InjectCommand<usize, RolledRow>>) -> bool {
        let mut was_changed = false;
        match self {
            FormList::RollEach(rows) => {
                for command in commands {
                    match command {
                        InjectCommand::Add(row) => {
                            rows.push(row.clone());
                            was_changed = true;
                        },
                        InjectCommand::Insert(idx, row) => {
                            if *idx < rows.len() {
                                rows.insert(*idx, row.clone());
                                was_changed = true;
                            }
                        },
                        InjectCommand::Remove(idx) => {
                            if *idx < rows.len() {
                                rows.remove(*idx);
                                was_changed = true;
                            }
                        },
                    }
                }
                was_changed
            },
            _ => false,
        }
    }

    pub fn inject_union(&mut self, commands: &Vec<InjectCommand<String, String>>) -> bool {
        let mut was_changed = false;
        match self {
            FormList::Union(rows) => {
                for command in commands {
                    was_changed |= match command {
                        InjectCommand::Add(row) => {
                           rows.insert(row.clone())
                        },
                        InjectCommand::Insert(_, _) => {
                            panic!("Union FormLists do not support the Insert command");
                        },
                        InjectCommand::Remove(row) => {
                            rows.remove(row)
                        },
                    };
                }
                was_changed
            },
            _ => false,
        }
    }

    pub fn inject_weighted(&mut self, commands: &Vec<InjectCommand<String, WeightedRow>>) -> bool {
        let mut was_changed = false;
        match self {
            FormList::Weighted(rows) => {
                for command in commands {
                    match command {
                        InjectCommand::Add(_) => {
                            panic!("Weighted FormLists do not support the Add command");
                        },
                        InjectCommand::Insert(asset, row) => {
                            rows.insert(asset.clone(), row.clone());
                            was_changed = true;
                        },
                        InjectCommand::Remove(asset) => {
                            was_changed |= rows.remove(asset).is_some();
                        },
                    }
                }
                was_changed
            },
            _ => false,
        }
    }
}
impl LuaUserData for FormList {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, this| match this {
            FormList::RollEach(_) => Ok("roll_each".to_string()),
            FormList::Union(_)    => Ok("union".to_string()),
            FormList::Weighted(_) => Ok("weighted".to_string()),
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, key: LuaValue| {
            match this {
                FormList::Weighted(m) => {
                    let asset = String::from_lua(key, lua)?;
                    if let Some(row) = m.get(&asset) {
                        let table = lua.create_table()?;
                        table.set("weight", row.weight)?;
                        table.set("vars", row.vars.0.clone())?;
                        Ok(Some(table))
                    } else { Ok(None) }
                },
                FormList::RollEach(rows) => {
                    let idx = i64::from_lua(key, lua)? - 1;
                    if idx < 0 || idx >= rows.len() as i64 {
                        Err(LuaError::RuntimeError(format!("index out of bounds for RollEach Formlist (was {}, max: {})", idx + 1, rows.len() + 1)))
                    } else {
                        let idx = idx as usize;
                        let row = &rows[idx];
                        let table = lua.create_table()?;
                        table.set("asset", row.asset.clone())?;
                        table.set("chance", row.chance)?;
                        table.set("vars", row.vars.0.clone())?;
                        Ok(Some(table))
                    }
                },
                FormList::Union(_) => Err(LuaError::RuntimeError(format!("Unable to index Union formlist"))),
            }
        });

        methods.add_meta_method_mut(LuaMetaMethod::NewIndex, |lua, this, (key, value): (LuaValue, LuaTable)| {
            match this {
                FormList::Weighted(rows) => {
                    let asset = String::from_lua(key, lua)?;
                    if let Some(row) = rows.get(&asset) {
                        // we already have an entry, so any missing fields can be assumed to not be updated
                        let weight = value.get::<_, Option<f32>>("weight")?.unwrap_or(row.weight);
                        let vars = LuaScriptVars(value.get::<_, Option<_>>("vars")?.unwrap_or_else(|| row.vars.0.clone()));
                        rows.insert(asset, WeightedRow { weight, vars });
                    } else {
                        let weight = value.get("weight")?;
                        let vars = LuaScriptVars(value.get("vars")?);
                        rows.insert(asset, WeightedRow { weight, vars });
                    }
                    Ok(())
                },
                FormList::RollEach(rows) => {
                    let idx = i64::from_lua(key, lua)? - 1;
                    if idx < 0 || idx >= rows.len() as i64 {
                        Err(LuaError::RuntimeError(format!("index out of bounds for RollEach Formlist (was {}, max: {})", idx + 1, rows.len() + 1)))
                    } else {
                        let idx = idx as usize;
                        let row = &rows[idx];
                        let asset = value.get::<_, Option<String>>("asset")?.unwrap_or_else(|| row.asset.clone());
                        let chance = value.get::<_, Option<f32>>("chance")?.unwrap_or(row.chance);
                        let vars = LuaScriptVars(value.get::<_, Option<_>>("vars")?.unwrap_or_else(|| row.vars.0.clone()));
                        rows[idx] = RolledRow { asset, chance, vars };
                        Ok(())
                    }
                },
                FormList::Union(_) => Err(LuaError::RuntimeError(format!("Unable to set index for Union formlist"))),
            }
        });

        methods.add_method_mut("add", |lua, this, value: LuaValue| {
            match this {
                FormList::Weighted(rows) => {
                    let table = LuaTable::from_lua(value, lua)?;
                    let asset = table.get("asset")?;
                    let weight = table.get("weight")?;
                    let vars   = LuaScriptVars(table.get("vars")?);
                    rows.insert(asset, WeightedRow { weight, vars });
                },
                FormList::RollEach(rows) => {
                    let table = LuaTable::from_lua(value, lua)?;
                    let asset = table.get("asset")?;
                    let chance = table.get("chance")?;
                    let vars   = LuaScriptVars(table.get("vars")?);
                    rows.push(RolledRow { asset, chance, vars });
                },
                FormList::Union(rows) => {
                    rows.insert(String::from_lua(value, lua)?);
                },
            }
            Ok(())
        });

        methods.add_method_mut("eval", |lua, this, ()| {
            match this {
                FormList::Weighted(rows) => {
                    let sum: f32 = rows.values().map(|r| r.weight).sum();
                    let r = random_range(0., sum);
                    let mut total = 0.;
                    for (asset, row) in rows.iter() {
                        total += row.weight;
                        if r <= total {
                            let table = lua.create_table()?;
                            table.set("asset", asset.clone())?;
                            if !row.vars.0.is_empty() {
                                table.set("vars", row.vars.0.clone())?;
                            }
                            return Ok(LuaValue::Table(table));
                        }
                    }
                    Ok(LuaValue::Nil)
                },
                FormList::RollEach(rows) => {
                    let mut res = Vec::new();
                    for row in rows.iter() {
                        if random_range(0., 1.) <= row.chance {
                            let table = lua.create_table()?;
                            table.set("asset", row.asset.clone())?;
                            if !row.vars.0.is_empty() {
                                table.set("vars", row.vars.0.clone())?;
                            }
                            res.push(table);
                        }
                    }
                    res.to_lua(lua)
                },
                FormList::Union(rows) => {
                    let table = lua.create_table()?;
                    for row in rows.iter() {
                        table.set(row.clone(), true)?;
                    }
                    Ok(LuaValue::Table(table))
                },
            }
        });

        methods.add_method_mut("remove", |lua, this, key: LuaValue| {
            match this {
                FormList::Weighted(rows) => {
                    let asset = String::from_lua(key, lua)?;
                    rows.remove(&asset);
                },
                FormList::RollEach(rows) => {
                    let idx = i64::from_lua(key, lua)? - 1;
                    if !(idx <= 0 || idx >= rows.len() as i64) {
                        rows.remove(idx as usize);
                    }
                },
                FormList::Union(rows) => {
                    let asset = String::from_lua(key, lua)?;
                    rows.remove(&asset);
                },
            }
            Ok(())
        });
    }
}
impl LuaMod for FormList {
    fn mod_name() -> &'static str { "FormList" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("handle_of", lua.create_function(|ctx, entity: LuaEntity| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            if let Some(handle) = w.get::<Handle<FormList>>(entity.inner()?) {
                Ok(Some(LuaHandle::from(handle.clone())))
            } else { Ok(None) }
        })?)?;
        table.set("load", lua.create_function(|lua, path: String| {
            let path = fix_missing_extension::<FormListLoader>(path);
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let asset_server = w.get_resource::<AssetServer>().unwrap();
            let handle: Handle<FormList> = asset_server.load(&path);
            Ok(LuaHandle::from(handle))
        })?)?;
        table.set("new_roll_each", lua.create_function(|_ctx, ()| {
            Ok(FormList::RollEach(Vec::new()))
        })?)?;
        table.set("new_union", lua.create_function(|_ctx, ()| {
            Ok(FormList::Union(HashSet::new()))
        })?)?;
        table.set("new_weighted", lua.create_function(|_ctx, ()| {
            Ok(FormList::Weighted(HashMap::new()))
        })?)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct FormListLoader;

impl AssetLoader for FormListLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let list: FormList = ron::de::from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(list));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["list.ron"]
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum InjectCommand<K, R> {
    Add(R),
    Insert(K, R),
    Remove(K),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InjectCommands;
impl LuaMod for InjectCommands {
    fn mod_name() -> &'static str { "Inject" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("new_roll_each", lua.create_function(|_ctx, ()| {
            Ok(InjectRollEach(Vec::new()))
        })?)?;
        table.set("new_union", lua.create_function(|_ctx, ()| {
            Ok(InjectUnion(Vec::new()))
        })?)?;
        table.set("new_weighted", lua.create_function(|_ctx, ()| {
            Ok(InjectWeighted(Vec::new()))
        })?)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InjectRollEach(pub Vec<InjectCommand<usize,  RolledRow>>);
impl LuaUserData for InjectRollEach {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, _| Ok("roll_each".to_string()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_method_mut("add", |_, this, table: LuaTable| {
            let asset  = table.get("asset")?;
            let chance = table.get("chance")?;
            let vars   = LuaScriptVars(table.get("vars")?);
            this.0.push(InjectCommand::Add(RolledRow { asset, chance, vars }));
            Ok(())
        });
        methods.add_method("apply", |_, this, mut formlist: FormList| {
            let is_success = formlist.inject_roll_each(&this.0);
            Ok((is_success, formlist))
        });
        methods.add_method_mut("insert", |_, this, (index, table): (usize, LuaTable)| {
            let asset  = table.get("asset")?;
            let chance = table.get("chance")?;
            let vars   = LuaScriptVars(table.get("vars")?);
            this.0.push(InjectCommand::Insert(index, RolledRow { asset, chance, vars }));
            Ok(())
        });
        methods.add_method_mut("remove", |_, this, key| {
            this.0.push(InjectCommand::Remove(key));
            Ok(())
        });
        
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InjectUnion(pub Vec<InjectCommand<String, String>>);
impl LuaUserData for InjectUnion {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, _| Ok("union".to_string()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_method_mut("add", |_, this, entry: String| {
            this.0.push(InjectCommand::Add(entry));
            Ok(())
        });
        methods.add_method("apply", |_, this, mut formlist: FormList| {
            let is_success = formlist.inject_union(&this.0);
            Ok((is_success, formlist))
        });
        methods.add_method_mut("remove", |_, this, key| {
            this.0.push(InjectCommand::Remove(key));
            Ok(())
        });
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InjectWeighted(pub Vec<InjectCommand<String, WeightedRow>>);
impl LuaUserData for InjectWeighted {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, _| Ok("weighted".to_string()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_method_mut("add", |_, this, table: LuaTable| {
            let asset = table.get("asset")?;
            let weight = table.get("weight")?;
            let vars   = LuaScriptVars(table.get("vars")?);
            this.0.push(InjectCommand::Insert(asset, WeightedRow { weight, vars }));
            Ok(())
        });
        methods.add_method("apply", |_, this, mut formlist: FormList| {
            let is_success = formlist.inject_weighted(&this.0);
            Ok((is_success, formlist))
        });
        methods.add_method_mut("insert", |_, this, (asset, table): (String, LuaTable)| {
            let weight = table.get("weight")?;
            let vars   = LuaScriptVars(table.get("vars")?);
            this.0.push(InjectCommand::Insert(asset, WeightedRow { weight, vars }));
            Ok(())
        });
        methods.add_method_mut("remove", |_, this, key| {
            this.0.push(InjectCommand::Remove(key));
            Ok(())
        });
    }
}
