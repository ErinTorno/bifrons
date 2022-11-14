use std::collections::HashMap;

use bevy_mod_scripting::lua::api::bevy::LuaWorld;
use mlua::prelude::*;
use super::LuaMod;

#[derive(Debug, Default)]
pub struct Registry {
    pub keys: HashMap<String, LuaRegistryKey>,
}
impl LuaMod for Registry {
    fn mod_name() -> &'static str { "Registry" }
    fn register_defs(ctx: &Lua, table: &mut LuaTable) -> Result<(), LuaError> {
        table.set("alloc_if_new", ctx.create_function(|lua, (name, function): (String, LuaFunction)| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut registry = w.get_resource_mut::<Registry>().unwrap();
            if !registry.keys.contains_key(&name) {
                let val: LuaValue = function.call(())?;
                let key = lua.create_registry_value(val)?;
                registry.keys.insert(name, key);
            }
            Ok(())
        })?)?;
        table.set("contains", ctx.create_function(|lua, name: String| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let registry = w.get_resource::<Registry>().unwrap();
            Ok(registry.keys.contains_key(&name))
        })?)?;
        table.set("free", ctx.create_function(|lua, name: String| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut registry = w.get_resource_mut::<Registry>().unwrap();
            if let Some(key) = registry.keys.remove(&name) {
                lua.remove_registry_value(key)?;
                Ok(true)
            } else {
                Ok(false)
            }
        })?)?;
        table.set("get", ctx.create_function(|lua, name: String| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let registry = w.get_resource::<Registry>().unwrap();
            if let Some(key) = registry.keys.get(&name) {
                Ok(Some(lua.registry_value::<LuaValue>(key)?))
            } else { Ok(None) }
        })?)?;
        table.set("replace", ctx.create_function(|lua, (name, val): (String, LuaValue)| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut registry = w.get_resource_mut::<Registry>().unwrap();
            if let Some(key) = registry.keys.get_mut(&name) {
                lua.replace_registry_value(key, val)?;
            }
            Ok(())
        })?)?;
        table.set("update", ctx.create_function(|lua, (name, function): (String, LuaFunction)| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut registry = w.get_resource_mut::<Registry>().unwrap();
            if let Some(key) = registry.keys.get_mut(&name) {
                let val: LuaValue = function.call(lua.registry_value::<LuaValue>(key)?)?;
                lua.replace_registry_value(key, val)?;
            }
            Ok(())
        })?)?;
        Ok(())
    }
}