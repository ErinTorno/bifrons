use std::{collections::HashMap, str::FromStr};

use bevy::{prelude::*, reflect::TypeUuid, utils::BoxedFuture, asset::*};
use bevy_mod_scripting::lua::api::bevy::LuaWorld;
use lazy_static::lazy_static;
use mlua::prelude::*;
use serde::{de, Serializer};
use serde::{de::*, Deserialize, Serialize};

use crate::scripting::{color::RgbaColor, LuaHandle, ScriptVar};
use crate::util::IntoHex;

#[derive(Clone, Component, Debug, Eq, Hash, PartialEq)]
pub enum DynColor {
    Background,
    Custom(RgbaColor),
    Named(String),
}
impl FromStr for DynColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "background" {
            Ok(DynColor::Background)
        } else if '#' == s.chars().next().ok_or_else(|| "Color string cannot be empty".to_string())? {
            Color::hex(&s[1..].to_string())
                .map(RgbaColor::from)
                .map(DynColor::Custom)
                .map_err(|e| format!("{}", e))
        } else {
            Ok(DynColor::Named(s.to_string()))
        }
    }
}
impl<'de> Deserialize<'de> for DynColor {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let s: String = Deserialize::deserialize(d)?;
        DynColor::from_str(&s).map_err(|e| de::Error::custom(e))
    }
}
impl Serialize for DynColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let s = match self {
            DynColor::Background  => "background".to_string(),
            DynColor::Custom(c)   => Color::from(*c).into_hex(),
            DynColor::Named(name) => name.clone(),
        };
        serializer.serialize_str(s.as_str())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ColorMiss {
    #[default]
    Identity,
    Clamp,
    Fn(String), // path/to/script.lua#my_function|params=123,are=like,this=true
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaletteConfig {
    #[serde(default)]
    base:    Option<String>,
    #[serde(default)]
    on_miss: ColorMiss,
    colors:  HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, TypeUuid)]
#[uuid = "f1c78ac4-576b-4504-85b6-96e5bf3bd9e1"]
pub struct Palette {
    base:    Option<String>,
    on_miss: ColorMiss,
    colors:  HashMap<String, Color>,
}
lazy_static! {
    static ref DEFAULT_PALETTE: Palette = {
        let bytes = include_bytes!("../../assets/palettes/default.palette.ron");
        ron::de::from_bytes(bytes).unwrap()
    };
}
impl Default for Palette {
    fn default() -> Self {
        DEFAULT_PALETTE.clone()
    }
}
impl<'de> Deserialize<'de> for Palette {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let mut colors = HashMap::new();
        let mut color_refs = HashMap::new();
        let config: PaletteConfig = Deserialize::deserialize(d)?;
        for (k, full) in config.colors {
            match full.chars().next().ok_or_else(|| de::Error::custom("Color string cannot be empty"))? {
                '#' => {
                    let c = Color::hex(&full[1..].to_string()).map_err(|e| de::Error::custom(e))?;
                    colors.insert(k, c);
                },
                _ => { color_refs.insert(k, full); },
            }
        }
        for (k, name) in color_refs {
            let c = colors.get(&name).ok_or_else(|| {
                de::Error::custom(format!("No concrete color defined for {}, required by alias {}", name, k))
            })?;
            colors.insert(k, c.clone());
        }
        Ok(Palette {
            base: config.base.clone(),
            colors,
            on_miss: config.on_miss.clone(),
        })
    }
}
impl LuaUserData for Palette {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
    }
}

#[derive(Default)]
pub struct PaletteLoader;

impl AssetLoader for PaletteLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let palette: Palette = ron::de::from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(palette));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["palette.ron"]
    }
}

#[derive(Clone, Debug, Default)]
pub struct CurrentPalette(pub Handle<Palette>);

#[derive(Clone, Debug, Default)]
pub struct ColorCache(pub HashMap<Handle<Palette>, HashMap<DynColor, RgbaColor>>);
impl ColorCache {
    pub fn color(&mut self, handle: &Handle<Palette>, color: &DynColor) -> Option<RgbaColor> {
        let palette = self.palette(handle);
        palette.get(color).cloned()
    }

    pub fn palette(&mut self, handle: &Handle<Palette>) -> &mut HashMap<DynColor, RgbaColor> {
        if !self.0.contains_key(handle) {
            self.0.insert(handle.clone(), HashMap::new());
        }
        self.0.get_mut(handle).unwrap()
    }
}

pub struct LuaColorCtx {
    pub base_color: RgbaColor,
    pub color: RgbaColor,
    pub config: HashMap<String, ScriptVar>,
    pub handle: Handle<Palette>,
}
impl LuaUserData for LuaColorCtx {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) where 'lua: 'lua {
        fields.add_field_method_get("base_color", |_, this| Ok(this.base_color));
        fields.add_field_method_get("color", |_, this| Ok(this.color));
        fields.add_field_method_set("color", |_, this, new_color| Ok(this.color = new_color));
        fields.add_field_method_get("config", |_, this| Ok(this.config.clone()));
        fields.add_field_method_get("handle", |_, this| Ok(LuaHandle::from(this.handle.clone())));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method_mut(LuaMetaMethod::Call, |_, this, color: RgbaColor| {
            this.color = color;
            Ok(())
        });
        methods.add_meta_method(LuaMetaMethod::Close, |ctx, this, ()| {
            info!("LuaColorCtx::Close");
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut cache = w.get_resource_mut::<ColorCache>().unwrap();
            let map = cache.palette(&this.handle);
            map.insert(DynColor::Custom(this.base_color), this.color);
            Ok(())
        });
    }
}