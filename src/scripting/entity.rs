use std::collections::HashMap;
use ::std::sync::Mutex;

use bevy::{prelude::*, reflect::*};
use bevy_mod_scripting::{prelude::*, lua::api::bevy::{LuaEntity, LuaWorld}, api::ScriptRef};
use mlua::Lua;

use crate::data::{prefab::Tags};

use super::{init_luamod, LuaMod};

#[derive(Default)]
pub struct EntityAPIProvider;

impl APIProvider for EntityAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_entity_lua(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<LuaQuery>(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

fn attach_entity_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    let table = ctx.create_table()?;
    table.set("hide", ctx.create_function(|ctx, entity: LuaEntity| {
        if let Some(mut ent_mut) = ctx.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity_mut(entity.inner()?) {
            ent_mut.insert(Visibility { is_visible: false });
        }
        Ok(())
    })?)?;
    table.set("set_visible", ctx.create_function(|ctx, (entity, is_visible): (LuaEntity, bool)| {
        if let Some(mut ent_mut) = ctx.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity_mut(entity.inner()?) {
            ent_mut.insert(Visibility { is_visible });
        }
        Ok(())
    })?)?;
    table.set("show", ctx.create_function(|ctx, entity: LuaEntity| {
        if let Some(mut ent_mut) = ctx.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity_mut(entity.inner()?) {
            ent_mut.insert(Visibility { is_visible: true });
        }
        Ok(())
    })?)?;
    table.set("tags", ctx.create_function(|ctx, entity: LuaEntity| {
        if let Some(ent) = ctx.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity(entity.inner()?) {
            let v: Option<HashMap<String, bool>> = ent.get::<Tags>().map(|t| t.0.iter().map(|s| (s.clone(), true)).collect());
            Ok(v)
        } else { Ok(None) }
    })?)?;
    ctx.globals().set("Entity", table)?;
    Ok(())
}

/* ******* */
/* Queries */
/* ******* */

#[derive(Clone)]
pub struct LuaQuery {
    name:    Option<String>,
    with:    Vec<String>,
    without: Vec<String>,
}
impl LuaQuery {
    fn get_type_registries<F>(&self, field: F, world: &LuaWorld) -> Result<Vec<TypeRegistration>, LuaError> where F: Fn(&LuaQuery) -> &Vec<String> {
        let w = world.read();
        let registry = w.get_resource::<TypeRegistry>().unwrap().read();
        let mut types = Vec::new();
        for type_name in field(self) {
            if let Some(reg) = registry.get_with_short_name(type_name)
                .or_else(|| registry.get_with_name(type_name))
                .map(|registration| registration.clone()) {
                types.push(reg);
            } else {
                return Err(LuaError::RuntimeError(format!("Type {} doesn't exist or wasn't registered", type_name)));
            }
        }
        Ok(types)
    }
}

impl LuaUserData for LuaQuery {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_: &mut F) {
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("Query {{with = {:?}, without = {:?}}}", this.with, this.without)));
        methods.add_method("entities", |_ctx, this, world: LuaWorld| {
            let with_types    = this.get_type_registries(|q| &q.with,    &world)?;
            let without_types = this.get_type_registries(|q| &q.without, &world)?;

            let mut entities = Vec::new();
            let mut w = world.write();
            for entity in w.query::<Entity>().iter(&*w) {
                let entity: Entity = entity;
                let mut matches = true;
                if let Some(name) = &this.name {
                    matches = if let Some(entity_name) = w.get_entity(entity).and_then(|e| e.get::<Name>()) {
                        entity_name.as_str() == name.as_str()
                    } else { false }
                }
                if matches {
                    for comp_type in &without_types {
                        let component_data = comp_type.data::<ReflectComponent>().ok_or_else(|| {
                            LuaError::RuntimeError(format!("Type {} doesn't exist or wasn't registered", comp_type.short_name()))
                        })?;
        
                        if component_data.reflect(&w, entity).is_some() {
                            matches = false;
                            break;
                        }
                    }
                }
                if matches {
                    for comp_type in &with_types {
                        let component_data = comp_type.data::<ReflectComponent>().ok_or_else(|| {
                            LuaError::RuntimeError(format!("Type {} doesn't exist or wasn't registered", comp_type.short_name()))
                        })?;
        
                        if component_data.reflect(&w, entity).is_none() {
                            matches = false;
                            break;
                        }
                    }
                    if matches { entities.push(LuaEntity::new(entity)) }
                }
            }
            Ok(entities)
        });
        methods.add_function_mut("with", |_, (this, typename): (LuaAnyUserData, String)| {
            let mut this: LuaQuery = this.take()?;
            this.with.push(typename);
            Ok(this)
        });
        methods.add_function_mut("without", |_, (this, typename): (LuaAnyUserData, String)| {
            let mut this: LuaQuery = this.take()?;
            this.without.push(typename);
            Ok(this)
        });
    }
}
impl LuaMod for LuaQuery {
    fn mod_name() -> &'static str { "Query" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("named", lua.create_function(|_ctx, name| {
            Ok(LuaQuery {
                name:    Some(name),
                with:    Vec::new(),
                without: Vec::new(),
            })
        })?)?;
        table.set("new", lua.create_function(|_ctx, ()| {
            Ok(LuaQuery {
                name:    None,
                with:    Vec::new(),
                without: Vec::new(),
            })
        })?)?;
        Ok(())
    }
}