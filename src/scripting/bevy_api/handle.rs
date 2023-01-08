use std::collections::HashMap;

use bevy::{prelude::*};
use bevy_inspector_egui::Inspectable;
use mlua::prelude::*;
use parking_lot::MappedRwLockReadGuard;
use serde::{Deserialize, Serialize};

use crate::{data::{formlist::FormList, palette::Palette, lua::{LuaWorld, LuaScript}, material::TexMatInfo, level::{Level, LoadedLevel, LoadedLevelCache}}, scripting::{bevy_api::LuaEntity, ui::{self, font::UIFont}}};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct AssetEventKey {
    pub entity:    Option<Entity>,
    pub handle:    LuaHandle,
    pub script_id: u32,
}

#[derive(Debug, Default, Resource)]
pub struct LuaAssetEventRegistry {
    pub keys:          HashMap<String, LuaRegistryKey>,
    pub on_asset_load: HashMap<AssetEventKey, LuaRegistryKey>,
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, Eq, Inspectable, PartialEq, Serialize)]
pub enum AssetKind {
    Font,
    FormList,
    Image,
    Level { is_loaded: bool },
    Material,
    Palette,
    Script,
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
            AssetKind::Font        => asset_server.get_handle_path(self.handle.typed_weak::<UIFont>()),
            AssetKind::FormList    => asset_server.get_handle_path(self.handle.typed_weak::<FormList>()),
            AssetKind::Image       => asset_server.get_handle_path(self.handle.typed_weak::<Image>()),
            AssetKind::Level {..}  => asset_server.get_handle_path(self.handle.typed_weak::<Level>()),
            AssetKind::Material    => asset_server.get_handle_path(self.handle.typed_weak::<StandardMaterial>()),
            AssetKind::Palette     => asset_server.get_handle_path(self.handle.typed_weak::<Palette>()),
            AssetKind::Script      => asset_server.get_handle_path(self.handle.typed_weak::<LuaScript>()),
            AssetKind::UiContainer => asset_server.get_handle_path(self.handle.typed_weak::<ui::elem::Container>()),
        }.map(|p| p.path().to_string_lossy().to_string())
    }
    
    pub fn get_image(&self) -> Option<Handle<Image>> {
        if let AssetKind::Image = self.kind {
            Some(self.handle.clone().typed())
        } else { None }
    }
    pub fn try_font(&self) -> Result<Handle<UIFont>, mlua::Error> {
        match self.kind {
            AssetKind::Font => Ok(self.handle.clone().typed()),
            _ => Err(mlua::Error::RuntimeError(format!("Unable to cast handle of kind {:?} to handle of Font", self.kind))),
        }
    }

    pub fn try_image(&self) -> Result<Handle<Image>, mlua::Error> {
        match self.kind {
            AssetKind::Image => Ok(self.handle.clone().typed()),
            _ => Err(mlua::Error::RuntimeError(format!("Unable to cast handle of kind {:?} to handle of Image", self.kind))),
        }
    }

    pub fn try_palette(&self) -> Result<Handle<Palette>, mlua::Error> {
        match self.kind {
            AssetKind::Palette => Ok(self.handle.clone().typed()),
            _ => Err(mlua::Error::RuntimeError(format!("Unable to cast handle of kind {:?} to handle of Palette", self.kind))),
        }
    }

    pub fn try_script(&self) -> Result<Handle<LuaScript>, mlua::Error> {
        match self.kind {
            AssetKind::Script => Ok(self.handle.clone().typed()),
            _ => Err(mlua::Error::RuntimeError(format!("Unable to cast handle of kind {:?} to handle of Script", self.kind))),
        }
    }

    pub fn try_ui_container(&self) -> Result<Handle<ui::elem::Container>, mlua::Error> {
        match self.kind {
            AssetKind::UiContainer => Ok(self.handle.clone().typed()),
            _ => Err(mlua::Error::RuntimeError(format!("Unable to cast handle of kind {:?} to handle of ui::elem::Container", self.kind))),
        }
    }
}
impl From<Handle<UIFont>> for LuaHandle {
    fn from(handle: Handle<UIFont>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Font }
    }
}
impl From<Handle<FormList>> for LuaHandle {
    fn from(handle: Handle<FormList>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::FormList }
    }
}
impl From<Handle<Image>> for LuaHandle {
    fn from(handle: Handle<Image>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Image }
    }
}
impl From<Handle<Level>> for LuaHandle {
    fn from(handle: Handle<Level>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Level { is_loaded: false } }
    }
}
impl From<Handle<LoadedLevel>> for LuaHandle {
    fn from(handle: Handle<LoadedLevel>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Level { is_loaded: true } }
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
impl From<Handle<LuaScript>> for LuaHandle {
    fn from(handle: Handle<LuaScript>) -> Self {
        LuaHandle { handle: handle.clone_untyped(), kind: AssetKind::Script }
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
            AssetKind::Font        => "font",
            AssetKind::FormList    => "formlist",
            AssetKind::Image       => "image",
            AssetKind::Level {..}  => "level",
            AssetKind::Material    => "material",
            AssetKind::Palette     => "palette",
            AssetKind::Script      => "script",
            AssetKind::UiContainer => "uicontainer",
        }));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        fn is_loaded(w: &MappedRwLockReadGuard<World>, this: &LuaHandle) -> Result<bool, LuaError> {
            match this.kind {
                AssetKind::Font => {
                    let assets = w.resource::<Assets<UIFont>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::FormList => {
                    let assets = w.resource::<Assets<FormList>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::Image => {
                    let assets = w.resource::<Assets<Image>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::Level { is_loaded } => {
                    Ok(if is_loaded {
                        let assets = w.resource::<Assets<LoadedLevel>>();
                        assets.contains(&this.handle.clone_weak().typed())
                    } else {
                        let ll_cache = w.resource::<LoadedLevelCache>();
                        if let Some(handle) = ll_cache.loaded_by_level.get(&this.handle.clone_weak().typed()) {
                            let assets = w.resource::<Assets<LoadedLevel>>();
                            assets.contains(handle)
                        } else { false }
                    })
                },
                AssetKind::Material => {
                    let assets = w.resource::<Assets<StandardMaterial>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::Palette => {
                    let assets = w.resource::<Assets<Palette>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::Script => {
                    let assets = w.resource::<Assets<LuaScript>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
                AssetKind::UiContainer => {
                    let assets = w.resource::<Assets<ui::elem::Container>>();
                    Ok(assets.contains(&this.handle.clone_weak().typed()))
                },
            }
        }

        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: LuaHandle| Ok(this == &that));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("#handle<{:?}>{{id = {:?}}}", this.kind, this.handle.id)));
        methods.add_method("get", |lua: &Lua, this: &LuaHandle, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            match this.kind {
                AssetKind::Font => Err(LuaError::RuntimeError("Cannot load Font assets into Lua".to_string())),
                AssetKind::FormList => {
                    let assets = w.get_resource::<Assets<FormList>>().unwrap();
                    if let Some(asset) = assets.get(&this.handle.clone().typed()) {
                        Ok(Some(asset.clone().to_lua(lua)?))
                    } else { Ok(None) }
                },
                AssetKind::Image      => Err(LuaError::RuntimeError("Cannot load Image assets into Lua".to_string())),
                AssetKind::Level { is_loaded } => {
                    let assets = w.resource::<Assets<LoadedLevel>>();
                    Ok(if is_loaded {
                        let assets = w.resource::<Assets<LoadedLevel>>();
                        if let Some(asset) = assets.get(&this.handle.clone().typed()) {
                            Some(asset.clone().to_lua(lua)?)
                        } else { None }
                    } else {
                        let ll_cache = w.resource::<LoadedLevelCache>();
                        if let Some(handle) = ll_cache.loaded_by_level.get(&this.handle.clone_weak().typed()) {
                            if let Some(asset) = assets.get(handle) {
                                Some(asset.clone().to_lua(lua)?)
                            } else { None }
                        } else { None }
                    })
                },
                AssetKind::Material   => {
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
                AssetKind::Script      => Err(LuaError::RuntimeError("Cannot load Script assets into Lua".to_string())),
                AssetKind::UiContainer => Err(LuaError::RuntimeError("Cannot load UiContainer assets into Lua".to_string())),
            }
        });
        methods.add_method("is_loaded", |lua, this, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            is_loaded(&w, this)
        });
        methods.add_method("on_load", |lua, this, f: LuaFunction| {
            let world = lua.globals().get::<_, LuaWorld>("world")?;
            if {
                let w = world.read();
                is_loaded(&w, this)?
            } {
                f.call(this.clone_weak())
            } else {
                let entity    = lua.globals().get::<_, Option<LuaEntity>>("entity")?.map(|e| e.0);
                let script_id = lua.globals().get::<_, u32>("script_id")?;
                let key       = AssetEventKey {
                    entity, script_id, handle: this.clone(),
                };
                let mut w = world.write();
                let mut registry = w.resource_mut::<LuaAssetEventRegistry>();
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
            let world = lua.globals().get::<_, LuaWorld>("world")?;
            let w = world.read();
            let asset_server = w.resource::<AssetServer>();
            Ok(this.get_path(asset_server))
        });
        methods.add_method("weak", |_, this, ()| Ok(this.clone_weak()));
    }
}