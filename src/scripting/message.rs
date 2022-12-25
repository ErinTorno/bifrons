use mlua::prelude::*;
use std::{default::default};

use crate::{data::lua::{LuaWorld, Recipient, ManyTransVars, ScriptVar, Hook, TransVar}, system::lua::{SharedInstances, LuaQueue, HookCall}};

use super::{ LuaMod, bevy_api::LuaEntity};

#[derive(Clone, Debug)]
pub struct MessageBuilder {
    pub hook_name: String,
    pub args:      ManyTransVars,
    pub recipient: Recipient,
    pub read_lock: bool,
}
impl LuaUserData for MessageBuilder {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("hook_name",   |_, this| Ok(this.hook_name.clone()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, ()| Ok(this.args.0.len()));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_function("add_arg", |_, (this, var): (LuaAnyUserData, TransVar)| {
            let mut this: MessageBuilder = this.take()?;
            this.args.0.push(var);
            Ok(this)
        });
        methods.add_function("read_lock", |_, this: LuaAnyUserData| {
            let mut this: MessageBuilder = this.take()?;
            this.read_lock = true;
            Ok(this)
        });
        methods.add_function("send", |lua, this: LuaAnyUserData| {
            let this: MessageBuilder = this.take()?;
            match this.recipient {
                Recipient::Entity(entity) => {
                    let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                    let mut w = world.write();
                    if let Some(mut ent) = w.get_entity_mut(entity) {
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
                        let si = w.get_resource::<SharedInstances>().unwrap();
                        si.by_path.get(&name).cloned()
                    } {
                        for entity in entities.keys() {
                            let mut w = world.write();
                            if let Some(mut ent) = w.get_entity_mut(*entity) {
                                if let Some(mut queue) = ent.get_mut::<LuaQueue>() {
                                    queue.calls.push(HookCall::next_frame(Hook {
                                        name: this.hook_name.clone(), args: this.args.clone()
                                    }));
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
        methods.add_function("to_entity", |_, (this, entity): (LuaAnyUserData, LuaEntity)| {
            let mut this: MessageBuilder = this.take()?;
            this.recipient = Recipient::Entity(entity.0);
            Ok(this)
        });
        methods.add_function("to_all", |_, this: LuaAnyUserData| {
            let mut this: MessageBuilder = this.take()?;
            this.recipient = Recipient::Everyone;
            Ok(this)
        });
        methods.add_function("to_script", |_, (this, script_name): (LuaAnyUserData, String)| {
            let mut this: MessageBuilder = this.take()?;
            this.recipient = Recipient::Script(script_name);
            Ok(this)
        });
    }
}
impl LuaMod for MessageBuilder {
    fn mod_name() -> &'static str { "Message" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("new", lua.create_function(|_ctx, hook_name| {
            Ok(MessageBuilder {
                hook_name,
                recipient: Recipient::NoOne,
                args: default(),
                read_lock: false,
            })
        })?)?;
        Ok(())
    }
}