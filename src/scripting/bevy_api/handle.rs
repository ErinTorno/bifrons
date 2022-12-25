use bevy::{prelude::*};
use bevy_inspector_egui::Inspectable;
use mlua::prelude::*;
use parking_lot::MappedRwLockReadGuard;
use serde::{Deserialize, Serialize};

use crate::{data::{formlist::FormList, palette::Palette, lua::LuaWorld, material::TexMatInfo, level::Level}, scripting::{registry::{Registry, AssetEventKey}, bevy_api::LuaEntity, ui}};

#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, Inspectable, PartialEq, Serialize)]
pub enum AssetKind {
    FormList,
    Image,
    Level,
    Material,
    Palette,
    UiContainer,
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
            AssetKind::Image    => asset_server.get_handle_path(self.handle.typed_weak::<Image>()),
            AssetKind::Level    => asset_server.get_handle_path(self.handle.typed_weak::<Level>()),
            AssetKind::Material => asset_server.get_handle_path(self.handle.typed_weak::<StandardMaterial>()),
            AssetKind::Palette  => asset_server.get_handle_path(self.handle.typed_weak::<Palette>()),
            AssetKind::UiContainer => asset_server.get_handle_path(self.handle.typed_weak::<ui::elem::Container>()),
        }.map(|p| p.path().to_string_lossy().to_string())
    }
    
    pub fn get_image(&self) -> Option<Handle<Image>> {
        if let AssetKind::Image = self.kind {
            Some(self.handle.clone().typed())
        } else { None }
    }

    pub fn try_image(&self) -> Result<Handle<Image>, mlua::Error> {
        match self.kind {
            AssetKind::Image => Ok(self.handle.clone().typed()),
            _ => Err(mlua::Error::RuntimeError(format!("Unable to cast handle of kind {:?} to handle of Image", self.kind))),
        }
    }

    pub fn try_ui_container(&self) -> Result<Handle<ui::elem::Container>, mlua::Error> {
        match self.kind {
            AssetKind::UiContainer => Ok(self.handle.clone().typed()),
            _ => Err(mlua::Error::RuntimeError(format!("Unable to cast handle of kind {:?} to handle of ui::elem::Container", self.kind))),
        }
    }
}
impl From<Handle<Image>> for LuaHandle {
    fn from(handle: Handle<Image>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Image }
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
impl From<Handle<ui::elem::Container>> for LuaHandle {
    fn from(handle: Handle<ui::elem::Container>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::UiContainer }
    }
}
impl LuaUserData for LuaHandle {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, this| Ok(match this.kind {
            AssetKind::FormList => "formlist",
            AssetKind::Image    => "image",
            AssetKind::Level    => "level",
            AssetKind::Material => "material",
            AssetKind::Palette  => "palette",
            AssetKind::UiContainer  => "uicontainer",
        }));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        fn is_loaded(w: &MappedRwLockReadGuard<World>, this: &LuaHandle) -> Result<bool, LuaError> {
            match this.kind {
                AssetKind::FormList => {
                    let assets = w.resource::<Assets<FormList>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::Image => {
                    let assets = w.resource::<Assets<Image>>();
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
                AssetKind::UiContainer => {
                    let assets = w.resource::<Assets<ui::elem::Container>>();
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
                AssetKind::Image => Err(LuaError::RuntimeError("Cannot load Image assets into Lua".to_string())),
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
                AssetKind::UiContainer => Err(LuaError::RuntimeError("Cannot load UiContainer assets into Lua".to_string())),
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