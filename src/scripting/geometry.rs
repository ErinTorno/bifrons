use ::std::sync::Mutex;

use bevy::prelude::{info, warn, error, Vec3};
use bevy_mod_scripting::prelude::*;
use mlua::Lua;
use serde::{Deserialize, Serialize};

use crate::{scripting::format_lua, data::geometry::{LightKind, Light, LightAnim}};

use super::{init_luamod, LuaMod};

#[derive(Default)]
pub struct GeometryAPIProvider;

impl APIProvider for GeometryAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        init_luamod::<Light>(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<LightAnim>(ctx).map_err(ScriptError::new_other)?;
        init_luamod::<LightKind>(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}