use bevy::{prelude::*};
use mlua::prelude::*;

use crate::data::{formlist::FormList, palette::Palette, lua::LuaWorld, material::TexMatInfo, level::Level};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetKind {
    FormList,
    Level,
    Material,
    Palette,
}

#[derive(Clone, Debug)]
pub struct LuaHandle {
    pub handle: HandleUntyped,
    pub kind:   AssetKind,
}
impl LuaHandle {
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
        LuaHandle { handle: handle.clone_weak_untyped(), kind: AssetKind::FormList }
    }
}
impl From<Handle<Level>> for LuaHandle {
    fn from(handle: Handle<Level>) -> Self {
        LuaHandle { handle: handle.clone_weak_untyped(), kind: AssetKind::Level }
    }
}
impl From<Handle<StandardMaterial>> for LuaHandle {
    fn from(handle: Handle<StandardMaterial>) -> Self {
        LuaHandle { handle: handle.clone_weak_untyped(), kind: AssetKind::Material }
    }
}
impl From<Handle<Palette>> for LuaHandle {
    fn from(handle: Handle<Palette>) -> Self {
        LuaHandle { handle: handle.clone_weak_untyped(), kind: AssetKind::Palette }
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
            match this.kind {
                AssetKind::FormList => {
                    let assets = w.get_resource::<Assets<FormList>>().unwrap();
                    Ok(assets.contains(&this.handle.clone().typed()))
                },
                AssetKind::Level => {
                    let assets = w.get_resource::<Assets<Level>>().unwrap();
                    Ok(assets.contains(&this.handle.clone().typed()))
                },
                AssetKind::Material => {
                    let assets = w.get_resource::<Assets<StandardMaterial>>().unwrap();
                    Ok(assets.contains(&this.handle.clone().typed()))
                },
                AssetKind::Palette => {
                    let assets = w.get_resource::<Assets<Palette>>().unwrap();
                    Ok(assets.contains(&this.handle.clone().typed()))
                },
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