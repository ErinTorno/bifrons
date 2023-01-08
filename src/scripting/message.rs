use mlua::prelude::*;
use std::{default::default, collections::HashSet};

use crate::{data::lua::{LuaWorld, Recipient, ManyTransVars, Hook, TransVar}, system::lua::{SharedInstances, LuaQueue, HookCall}, util::collections::Singleton};

use super::{ LuaMod, bevy_api::LuaEntity};

#[derive(Clone, Debug)]
pub struct MessageBuilder {
    pub hook_name: String,
    pub args:      ManyTransVars,
    pub recipient: Recipient,
}
impl LuaUserData for MessageBuilder {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("hook_name",   |_, this| Ok(this.hook_name.clone()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, ()| Ok(this.args.0.len()));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

        methods.add_function("attach", |_, (mut this, var): (MessageBuilder, TransVar)| {
            this.args.0.push(var);
            Ok(this)
        });
        methods.add_method("send", |lua, this, ()| {
            match &this.recipient {
                Recipient::Entity(entity) => {
                    let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                    let mut w = world.write();
                    if let Some(mut ent) = w.get_entity_mut(*entity) {
                        if let Some(mut queue) = ent.get_mut::<LuaQueue>() {
                            queue.calls.push(HookCall::next_frame(Hook {
                                name: this.hook_name.clone(), args: this.args.clone()
                            }));
                        }
                    }
                },
                Recipient::Everyone => {
                    let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                    let mut w = world.write();
                    let mut query = w.query::<&mut LuaQueue>();
                    for mut queue in query.iter_mut(&mut *w) {
                        queue.calls.push(HookCall::next_frame(Hook {
                            name: this.hook_name.clone(), args: this.args.clone()
                        }));
                    }
                },
                Recipient::NoOne => (),
                Recipient::Script(name) => {
                    let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                    if let Some(entities) = {
                        let w = world.read();
                        let si = w.resource::<SharedInstances>();
                        si.by_path.get(name).cloned()
                    } {
                        for (entity, script_id) in entities.iter() {
                            let mut w = world.write();
                            if let Some(mut ent) = w.get_entity_mut(*entity) {
                                if let Some(mut queue) = ent.get_mut::<LuaQueue>() {
                                    queue.calls.push(HookCall {
                                        script_ids: HashSet::singleton(*script_id),
                                        hook:       Hook { name: this.hook_name.clone(), args: this.args.clone() },
                                    });
                                }
                            }
                        }
                    } else {
                        return Err(LuaError::RuntimeError(format!("No scripts loaded from path {}", name)));
                    }
                },
            }
            Ok(())
        });
        methods.add_function("to_entity", |_, (mut this, entity): (MessageBuilder, LuaEntity)| {
            this.recipient = Recipient::Entity(entity.0);
            Ok(this)
        });
        methods.add_function("to_all", |_, mut this: MessageBuilder| {
            this.recipient = Recipient::Everyone;
            Ok(this)
        });
        methods.add_function("to_script", |_, (mut this, script_name): (MessageBuilder, String)| {
            this.recipient = Recipient::Script(script_name);
            Ok(this)
        });
    }
}
impl LuaMod for MessageBuilder {
    fn mod_name() -> &'static str { "Message" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("new", lua.create_function(|_, hook_name| {
            Ok(MessageBuilder {
                hook_name,
                recipient: Recipient::NoOne,
                args: default(),
            })
        })?)?;
        Ok(())
    }
}