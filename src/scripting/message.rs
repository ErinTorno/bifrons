use ::std::sync::Mutex;

use bevy_inspector_egui::Inspectable;
use bevy_mod_scripting::{prelude::*, lua::api::bevy::{LuaWorld, LuaEntity}};
use mlua::Lua;
use serde::{Deserialize, Serialize};

use super::{init_luamod, LuaMod, ManyScriptVars, ScriptVar};

#[derive(Default)]
pub struct MessageAPIProvider;

impl APIProvider for MessageAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        init_luamod::<MessageBuilder>(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct MessageQueue {
    pub events: Vec<LuaEvent<ManyScriptVars>>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Inspectable, PartialEq, Serialize)]
pub struct ScriptRef {
    pub id: u32,
}
impl LuaUserData for ScriptRef {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("#scriptref{{id = {}}}", this.id)));
    }
}

#[derive(Clone, Debug, Default)]
pub struct MessageBuilder {
    pub hook_name:  String,
    pub args:       ManyScriptVars,
    pub recipients: Recipients,
}
impl LuaUserData for MessageBuilder {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("hook_name",   |_, this| Ok(this.hook_name.clone()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, ()| Ok(this.args.0.len()));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_function("add_arg", |_, (this, var): (LuaAnyUserData, ScriptVar)| {
            let mut this: MessageBuilder = this.take()?;
            this.args.0.push(var);
            Ok(this)
        });
        methods.add_method("send", |ctx, this, ()| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut queue = w.get_resource_mut::<MessageQueue>().unwrap();
            queue.events.push(LuaEvent { hook_name: this.hook_name.clone(), args: this.args.clone(), recipients: this.recipients.clone() });
            Ok(())
        });
        methods.add_function("to_entity", |_, (this, entity): (LuaAnyUserData, LuaEntity)| {
            let entity = entity.inner()?;
            let mut this: MessageBuilder = this.take()?;
            this.recipients = Recipients::Entity(entity);
            Ok(this)
        });
        methods.add_function("to_all", |_, this: LuaAnyUserData| {
            let mut this: MessageBuilder = this.take()?;
            this.recipients = Recipients::All;
            Ok(this)
        });
        methods.add_function("to_script", |_, (this, script_name): (LuaAnyUserData, String)| {
            let mut this: MessageBuilder = this.take()?;
            this.recipients = Recipients::ScriptName(script_name);
            Ok(this)
        });
        methods.add_function("to_script_id", |_, (this, script_ref): (LuaAnyUserData, ScriptRef)| {
            let mut this: MessageBuilder = this.take()?;
            this.recipients = Recipients::ScriptID(script_ref.id);
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
                ..MessageBuilder::default()
            })
        })?)?;
        Ok(())
    }
}