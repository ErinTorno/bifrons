use std::{collections::HashMap, str::FromStr};

use bevy::ecs::system::SystemState;
use bevy::{prelude::*, reflect::TypeUuid, utils::BoxedFuture, asset::*};
use bevy_inspector_egui::prelude::*;
use lazy_static::lazy_static;
use mlua::prelude::*;
use palette::{Lcha, FromColor};
use palette::rgb::Rgba;
use serde::{de, Serializer};
use serde::{de::*, Deserialize, Serialize};

use crate::data::lua::ScriptVar;
use crate::scripting::LuaMod;
use crate::scripting::bevy_api::handle::LuaHandle;
use crate::scripting::{color::RgbaColor};
use crate::system::common::{fix_missing_extension};
use crate::system::lua::SharedInstances;
use crate::system::palette::LoadingPalette;
use crate::util::IntoHex;
use crate::util::serialize::ron_options;

use super::lua::{ManyScriptVars, Any2, LuaWorld, LuaReadable, InstanceRef};

#[derive(Clone, Component, Debug, Default, Eq, FromReflect, Hash, Inspectable, PartialEq, Reflect)]
pub enum DynColor {
    #[default]
    Background,
    Const(RgbaColor),
    Custom(RgbaColor),
    Named(String),
}
impl DynColor {
    pub const CONST_BLACK: DynColor = DynColor::Const(RgbaColor {r: 0., g: 0., b: 0., a: 1.});
    pub const CONST_WHITE: DynColor = DynColor::Const(RgbaColor {r: 1., g: 1., b: 1., a: 1.});

    pub fn placeholder(&self) -> Color {
        match self {
            DynColor::Background   => Color::BLACK,
            DynColor::Const(rgba)  => rgba.clone().into(),
            DynColor::Custom(rgba) => rgba.clone().into(),
            DynColor::Named(nm)    => DEFAULT_PALETTE.colors.get(nm).cloned().unwrap_or(Color::FUCHSIA),
        }
    }

    pub fn eval_lua(&self, lua: &Lua) -> RgbaColor {
        fn fuchsia_err() -> RgbaColor {
            error!("DynColor lua eval'ed but palettes asset was not yet loaded; using placeholder fuschia");
            RgbaColor::FUSCHIA
        }

        match self {
            DynColor::Background => {
                let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                let w = world.read();
                let loaded_pal = w.resource::<LoadedPalettes>();
                let palettes = w.resource::<Assets<Palette>>();
                if let Some(palette) = palettes.get(&loaded_pal.current_handle) {
                    palette.background.eval_lua(lua)
                } else { fuchsia_err() }
            },
            DynColor::Const(rgba) => *rgba,
            DynColor::Custom(_) => {
                let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                let mut w = world.write();
                
                let mut sys = SystemState::<(
                    Res<LoadedPalettes>,
                    Res<Assets<Palette>>,
                    ResMut<ColorCache>,
                )>::new(&mut w);
                let (loaded_pal, palettes, mut color_cache) = sys.get_mut(&mut w);

                if let Some(palette) = palettes.get(&loaded_pal.current_handle) {
                    color_cache.rgba(self, palette, lua)
                } else { fuchsia_err() }
            },
            DynColor::Named(name) => {
                let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                let w = world.read();
                let loaded_pal = w.resource::<LoadedPalettes>();
                let palettes = w.resource::<Assets<Palette>>();
                if let Some(palette) = palettes.get(&loaded_pal.current_handle) {
                    if let Some(color) = palette.colors.get(name) {
                        RgbaColor::from(color.clone())
                    } else {
                        palette.missing_color
                    }
                } else { fuchsia_err() }
            },
        }
    }
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
            DynColor::Const(c) => format!("{}!", Color::from(*c).into_hex()),
            DynColor::Custom(c)   => Color::from(*c).into_hex(),
            DynColor::Named(name) => name.clone(),
        };
        serializer.serialize_str(s.as_str())
    }
}
impl LuaUserData for DynColor {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("kind", |_, this| {
            Ok(match this {
                DynColor::Background => "background",
                DynColor::Const(_)   => "const",
                DynColor::Custom(_)  => "custom",
                DynColor::Named(_)   => "named",
            })
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(match this {
            DynColor::Background  => "background".to_string(),
            DynColor::Const(c) => format!("{}!", Color::from(*c).into_hex()),
            DynColor::Named(s)    => s.clone(),
            DynColor::Custom(c)   => c.into_hex(),
        }));
        // todo maybe if palette is passed in, we can eval according to that palette?
        methods.add_method("eval", |lua, this, ()| Ok(this.eval_lua(lua)));
        // and we can call() as a synonym for evaling
        methods.add_meta_method(LuaMetaMethod::Call, |lua, this, ()| Ok(this.eval_lua(lua)));
    }
}
impl LuaMod for DynColor {
    fn mod_name() -> &'static str { "Color" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("background", DynColor::Background.to_lua(lua)?)?;
        table.set("const", lua.create_function(|lua, any2: Any2<RgbaColor, DynColor>| {
            Ok(DynColor::Const(match any2 {
                Any2::A(rgba) => rgba,
                Any2::B(dcol) => dcol.eval_lua(lua),
            }))
        })?)?;
        table.set("named", lua.create_function(|_, name: String| {
            Ok(DynColor::Named(name))
        })?)?;
        table.set("custom", lua.create_function(|lua, any2: Any2<RgbaColor, DynColor>| {
            Ok(DynColor::Custom(match any2 {
                Any2::A(rgba) => rgba,
                Any2::B(dcol) => dcol.eval_lua(lua),
            }))
        })?)?;
        Ok(())
    }
}

#[derive(Clone, Component, Debug, Default, Deserialize, Inspectable, PartialEq, Reflect, Serialize)]
#[reflect(Component)]
pub struct SingleColored(pub DynColor);

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum ColorMiss {
    #[default]
    Identity,
    Clamp,
    Fn {
        file: String,
        function: String,
        #[serde(default)]
        params: ManyScriptVars,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaletteConfig {
    #[serde(default)]
    pub base:          Option<String>,
    #[serde(default)]
    pub on_miss:       ColorMiss,
    pub background:    DynColor,
    #[serde(default = "default_missing_color")]
    pub missing_color: RgbaColor,
    pub colors:        HashMap<String, String>,
}
pub fn default_missing_color() -> RgbaColor { RgbaColor::FUSCHIA }

#[derive(Clone, Debug, PartialEq, TypeUuid)]
#[uuid = "f1c78ac4-576b-4504-85b6-96e5bf3bd9e1"]
pub struct Palette {
    pub handle:        Handle<Palette>, // self-reference; makes some ColorCache stuff easier
    pub base:          Option<String>,
    pub on_miss:       ColorMiss,
    pub background:    DynColor,
    pub missing_color: RgbaColor,
    pub colors:        HashMap<String, Color>,
    pub colors_lch:    HashMap<String, Lcha>,
}
lazy_static! {
    static ref DEFAULT_PALETTE: Palette = {
        let bytes = include_bytes!("../../assets/palettes/default.palette.ron");
        ron_options().from_bytes(bytes).unwrap()
    };
}
impl Palette {
    pub fn get_script(&self) -> Option<&String> {
        match &self.on_miss {
            ColorMiss::Fn { file,.. } => Some(file),
            _ => None,
        }
    }
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
        if config.background == DynColor::Background {
            return Err(de::Error::custom("Background is defined recursively as \"background\""));
        }
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
        let colors_lch = colors.iter().map(|(k, v)| {
            let [r, g, b, a] = v.as_rgba_f32();
            let rgba: Rgba = Rgba::new(r, g, b, a);
            (k.clone(), Lcha::from_color(rgba))
        }).collect();
        Ok(Palette {
            handle: default(), // must be immediately set when to actual value palette is loaded
            base: config.base.clone(),
            colors,
            colors_lch,
            background: config.background,
            missing_color: config.missing_color,
            on_miss: config.on_miss.clone(),
        })
    }
}
impl LuaUserData for Palette {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(_methods: &mut M) {
    }
}
impl LuaMod for Palette {
    fn mod_name() -> &'static str { "Palette" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("change", lua.create_function(|lua, handle: LuaHandle| {
            let handle = handle.handle.clone().typed();
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let entity = {
                let loaded_palettes = w.resource::<LoadedPalettes>();
                loaded_palettes.by_handle.get(&handle).map(|st| st.entity).unwrap_or_else(|| {
                    w.spawn_empty().id()
                })
            };
            w.insert_resource(LoadingPalette { handle, entity });
            Ok(())
        })?)?;
        table.set("load", lua.create_function(|lua, path: String| {
            let path = fix_missing_extension::<PaletteLoader>(path);
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let asset_server = w.get_resource::<AssetServer>().unwrap();
            let handle: Handle<Palette> = asset_server.load(&path);
            Ok(LuaHandle::from(handle))
        })?)?;
        Ok(())
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
            let palette: Palette = ron_options().from_bytes(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(palette));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["palette.ron"]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadedPaletteState {
    pub entity:    Entity,
    pub script_id: u32,
}

#[derive(Clone, Debug, Resource)]
pub struct LoadedPalettes {
    pub current_handle: Handle<Palette>,
    pub current_state:  LoadedPaletteState,
    pub by_handle:      HashMap<Handle<Palette>, LoadedPaletteState>
}
impl LoadedPalettes {
    pub fn lua_instance<'a>(&self, shared_instances: &'a SharedInstances) -> Option<&'a InstanceRef> {
        match &shared_instances.instances.get(&self.current_state.script_id) {
            Some(inst) => match &inst.result {
                Ok(i) => Some(i),
                Err(_) => None,
            },
            None => Some(&shared_instances.collectivist),
        }
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct ColorCache(pub HashMap<Handle<Palette>, HashMap<DynColor, RgbaColor>>);
impl ColorCache {
    pub fn rgba<L>(&mut self, dyn_color: &DynColor, palette: &Palette, lua_readable: &L) -> RgbaColor where L: LuaReadable {
        let dyn_color = if let DynColor::Background = dyn_color { &palette.background } else { dyn_color };
        if let Some(rgba) = {
            let palette_map = self.palette(&palette.handle);
            if palette_map.is_empty() {
                for (name, color) in palette.colors.iter() {
                    palette_map.insert(DynColor::Named(name.clone()), color.clone().into());
                }
            }
            palette_map.get(&dyn_color)
        } {
            *rgba
        } else {
            match dyn_color {
                DynColor::Background => { unreachable!() },
                DynColor::Const(rgba) => *rgba,
                DynColor::Custom(rgba) => {
                    let rgba = *rgba;
                    let rgba = match &palette.on_miss {
                        ColorMiss::Clamp => {
                            // todo color match technology
                            palette.missing_color
                        },
                        ColorMiss::Identity => rgba,
                        ColorMiss::Fn { function, params,.. } => { // uh oh, danger!
                            let mut new_params = Vec::new();
                            new_params.push(ScriptVar::Color(rgba));
                            if !params.0.is_empty() {
                                let mut params = params.0.clone();
                                new_params.append(&mut params);
                            }
                            lua_readable.with_read(|lua| {
                                let globals = lua.globals();
                                if let Some(f) = globals.get::<_, Option<LuaFunction>>(function.as_str()).unwrap() {
                                    match f.call::<_, RgbaColor>(ManyScriptVars(new_params)) {
                                        Ok(c)    => c,
                                        Err(err) => {
                                            warn!("Function for on_miss Fn {} errored: {}", function, err);
                                            RgbaColor::FUSCHIA
                                        }
                                    }
                                } else {
                                    warn!("Function not found for on_miss Fn {}", function);
                                    RgbaColor::FUSCHIA
                                }
                            })
                        },
                    };
                    let palette_map = self.0.get_mut(&palette.handle).unwrap();
                    palette_map.insert(dyn_color.clone(), rgba);
                    rgba
                },
                DynColor::Named(name) => {
                    if let Some(color) = palette.colors.get(name) {
                        RgbaColor::from(*color)
                    } else if let Some(color) = DEFAULT_PALETTE.colors.get(name) {
                        let def_color = DynColor::Custom(color.clone().into());
                        let rgba = self.rgba(&def_color, palette, lua_readable);
                        let palette_map = self.0.get_mut(&palette.handle).unwrap();
                        palette_map.insert(dyn_color.clone(), rgba);
                        rgba
                    } else {
                        palette.missing_color
                    }
                },
            }
        }
    }

    pub fn palette(&mut self, handle: &Handle<Palette>) -> &mut HashMap<DynColor, RgbaColor> {
        if !self.0.contains_key(handle) {
            self.0.insert(handle.clone(), HashMap::new());
        }
        self.0.get_mut(handle).unwrap()
    }
}