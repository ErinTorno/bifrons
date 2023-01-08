use std::collections::HashMap;

use bevy::{prelude::*};
use mlua::prelude::*;

use crate::{data::{lua::LuaWorld, prefab::Tags}, scripting::LuaMod};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LuaEntity(pub Entity);
impl LuaEntity {
    pub fn new(entity: Entity) -> Self { LuaEntity(entity) }
}
impl LuaUserData for LuaEntity {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("index", |_, this| Ok(this.0.index()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("entity#{:?}", this.0)));

        methods.add_method("add_child", |lua, this, child: LuaEntity| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            if let Some(mut entity) = w.get_entity_mut(this.0) {
                entity.push_children(&[child.into()]);
            }
            Ok(())
        });
        methods.add_method("despawn", |lua, this, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let ent = w.entity_mut(this.0);
            ent.despawn();
            Ok(())
        });
        methods.add_method("hide", |lua, this, ()| {
            if let Some(mut ent_mut) = lua.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity_mut(this.0) {
                ent_mut.insert(Visibility { is_visible: false });
            }
            Ok(())
        });
        methods.add_method("show", |lua, this, ()| {
            if let Some(mut ent_mut) = lua.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity_mut(this.0) {
                ent_mut.insert(Visibility { is_visible: true });
            }
            Ok(())
        });
        methods.add_method("tags", |lua, this, ()| {
            if let Some(ent) = lua.globals().get::<_, LuaWorld>("world").unwrap().write().get_entity(this.0) {
                let v: Option<HashMap<String, bool>> = ent.get::<Tags>().map(|t| t.0.iter().map(|s| (s.clone(), true)).collect());
                Ok(v)
            } else { Ok(None) }
        });
    }
}
impl LuaMod for LuaEntity {
    fn mod_name() -> &'static str { "Entity" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("spawn", lua.create_function(|lua, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let ent = w.spawn(());
            Ok(LuaEntity(ent.id()))
        })?)?;
        Ok(())
    }
}
impl From<Entity> for LuaEntity {
    fn from(e: Entity) -> Self { LuaEntity(e) }
}
impl From<LuaEntity> for Entity {
    fn from(LuaEntity(e): LuaEntity) -> Self { e }
}