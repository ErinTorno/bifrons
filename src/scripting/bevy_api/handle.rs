use bevy::{prelude::*};
use mlua::prelude::*;
use parking_lot::MappedRwLockReadGuard;

use crate::{data::{formlist::FormList, palette::Palette, lua::LuaWorld, material::TexMatInfo, level::Level}, scripting::{registry::{Registry, AssetEventKey}, bevy_api::LuaEntity}};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum AssetKind {
    FormList,
    Level,
    Material,
    Palette,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct LuaHandle {
    pub handle: HandleUntyped,
    pub kind:   AssetKind,
}
impl LuaHandle {
    pub fn clone_weak(&self) -> LuaHandle {
        LuaHandle { handle: self.handle.clone_weak(), kind: self.kind }
    }

    pub fn get_path(&self, asset_server: &AssetServer) -> Option<String> {
        match self.kind {
            AssetKind::FormList => asset_server.get_handle_path(self.handle.typed_weak::<FormList>()),
            AssetKind::Level    => asset_server.get_handle_path(self.handle.typed_weak::<Level>()),
            AssetKind::Material => asset_server.get_handle_path(self.handle.typed_weak::<StandardMaterial>()),
            AssetKind::Palette  => asset_server.get_handle_path(self.handle.typed_weak::<Palette>()),
        }.map(|p| p.path().to_string_lossy().to_string())
    }
}
impl From<Handle<FormList>> for LuaHandle {
    fn from(handle: Handle<FormList>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::FormList }
    }
}
impl From<Handle<Level>> for LuaHandle {
    fn from(handle: Handle<Level>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Level }
    }
}
impl From<Handle<StandardMaterial>> for LuaHandle {
    fn from(handle: Handle<StandardMaterial>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Material }
    }
}
impl From<Handle<Palette>> for LuaHandle {
    fn from(handle: Handle<Palette>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Palette }
    }
}
impl LuaUserData for LuaHandle {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, this| Ok(match this.kind {
            AssetKind::FormList => "formlist",
            AssetKind::Level    => "level",
            AssetKind::Material => "material",
            AssetKind::Palette  => "palette",
        }));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        fn is_loaded(w: &MappedRwLockReadGuard<World>, this: &LuaHandle) -> Result<bool, LuaError> {
            match this.kind {
                AssetKind::FormList => {
                    let assets = w.resource::<Assets<FormList>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::Level => {
                    let assets = w.resource::<Assets<Level>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::Material => {
                    let assets = w.resource::<Assets<StandardMaterial>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::Palette => {
                    let assets = w.resource::<Assets<Palette>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
            }
        }

        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("#handle<{:?}>{{id = {:?}}}", this.kind, this.handle.id)));
        methods.add_method("get", |lua: &Lua, this: &LuaHandle, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            match this.kind {
                AssetKind::FormList => {
                    let assets = w.get_resource::<Assets<FormList>>().unwrap();
                    if let Some(asset) = assets.get(&this.handle.clone().typed()) {
                        Ok(Some(asset.clone().to_lua(lua)?))
                    } else { Ok(None) }
                },
                AssetKind::Level => Err(LuaError::RuntimeError("Cannot load Level assets into Lua; see the Level module".to_string())),
                AssetKind::Material => {
                    if let Some(tex_mat_info) = w.get_resource::<TexMatInfo>() {
                        if let Some(texmat) = tex_mat_info.materials.get(&this.handle.clone().typed()) {
                            Ok(Some(texmat.clone().to_lua(lua)?))
                        } else { Ok(None) }
                    } else {
                        Err(LuaError::RuntimeError(format!("TexMatInfo not found")))
                    }
                },
                AssetKind::Palette => {
                    let assets = w.get_resource::<Assets<Palette>>().unwrap();
                    if let Some(asset) = assets.get(&this.handle.clone().typed()) {
                        Ok(Some(asset.clone().to_lua(lua)?))
                    } else { Ok(None) }
                },
            }
        });
        methods.add_method("is_loaded", |lua, this, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            is_loaded(&w, this)
        });
        methods.add_method("on_load", |lua, this, f: LuaFunction| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            if {
                let w = world.read();
                is_loaded(&w, this)?
            } {
                f.call(this.clone_weak())
            } else {
                let entity    = lua.globals().get::<_, LuaEntity>("entity").unwrap().0;
                let script_id = lua.globals().get::<_, u32>("script_id").unwrap();
                let key       = AssetEventKey {
                    entity, script_id, handle: this.clone(),
                };
                let mut w = world.write();
                let mut registry = w.resource_mut::<Registry>();
                if let Some(reg_key) = registry.on_asset_load.get(&key) {
                    let mut v: Vec<LuaFunction> = lua.registry_value(reg_key)?;
                    v.push(f);
                    lua.replace_registry_value(reg_key, v)?;
                } else {
                    let v = vec![f];
                    let reg_key = lua.create_registry_value(v)?;
                    registry.on_asset_load.insert(key, reg_key);
                }
                Ok(())
            }
        });
        methods.add_method("path", |lua, this, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let asset_server = w.get_resource::<AssetServer>().unwrap();
            Ok(this.get_path(asset_server))
        });
    }
}