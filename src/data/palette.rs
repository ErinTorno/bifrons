use std::{collections::HashMap, str::FromStr};

use bevy::ecs::system::SystemState;
use bevy::{prelude::*, reflect::TypeUuid, utils::BoxedFuture, asset::*};
use bevy_inspector_egui::prelude::*;
use lazy_static::lazy_static;
use mlua::prelude::*;
use palette::{ColorDifference, Lcha, FromColor};
use serde::{de, Serializer};
use serde::{de::*, Deserialize, Serialize};

use crate::data::lua::ScriptVar;
use crate::scripting::{LuaMod, lua_to_string};
use crate::scripting::bevy_api::handle::LuaHandle;
use crate::scripting::{color::RgbaColor};
use crate::system::common::{fix_missing_extension};
use crate::system::lua::SharedInstances;
use crate::system::palette::LoadingPalette;
use crate::util::{IntoHex, RoughlyEq};
use crate::util::ron_options;

use super::lua::{ManyTransVars, LuaWorld, LuaReadable, InstanceRef, Any3, TransVar};

pub type LuaToDynColor = Any3<DynColor, RgbaColor, String>;

#[derive(Clone, Component, Debug, Default, Eq, FromReflect, Hash, Inspectable, PartialEq, Reflect)]
pub enum DynColor {
    #[default]
    Background,
    Const(RgbaColor),
    Custom(RgbaColor),
    Named(String),
}
impl DynColor {
    pub const CONST_BLACK:       DynColor = DynColor::Const(RgbaColor::BLACK);
    pub const CONST_FUCHSIA:     DynColor = DynColor::Const(RgbaColor::FUCHSIA);
    pub const CONST_TRANSPARENT: DynColor = DynColor::Const(RgbaColor::TRANSPARENT);
    pub const CONST_WHITE:       DynColor = DynColor::Const(RgbaColor::WHITE);

    pub fn placeholder(&self) -> Color {
        match self {
            DynColor::Background   => Color::BLACK,
            DynColor::Const(rgba)  => rgba.clone().into(),
            DynColor::Custom(rgba) => rgba.clone().into(),
            DynColor::Named(nm)    => DEFAULT_PALETTE.colors.get(nm).cloned().unwrap_or(RgbaColor::FUCHSIA).into(),
        }
    }

    pub fn eval_lua_current(&self, lua: &Lua) -> Result<RgbaColor, mlua::Error> {
        let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
        let w = world.read();
        let loaded_pal = w.resource::<LoadedPalettes>();
        self.eval_lua(&loaded_pal.current_handle, lua)
    }

    pub fn eval_lua(&self, handle: &Handle<Palette>, lua: &Lua) -> Result<RgbaColor, mlua::Error> {
        match self {
            DynColor::Const(rgba) => Ok(*rgba),
            _ => {
                let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                let mut w = world.write();
                
                let mut sys = SystemState::<(
                    Res<Assets<Palette>>,
                    ResMut<ColorCache>,
                )>::new(&mut w);
                let (palettes, mut color_cache) = sys.get_mut(&mut w);
                palettes.get(handle)
                    .ok_or_else(|| mlua::Error::RuntimeError("DynColor lua eval'ed but palettes asset was not yet loaded".to_string()))
                    .map(|palette| color_cache.rgba(self, palette, lua))
            },
        }
    }

    pub fn from_any(any: LuaToDynColor) -> DynColor {
        match any {
            Any3::A(c) => { c },
            Any3::B(c) => { DynColor::Custom(c) },
            Any3::C(c) => { DynColor::Named(c) },
        }
    }


    pub fn lua_to_string(&self) -> String {
        match self {
            DynColor::Background   => "Color.background".to_string(),
            DynColor::Const(rgba)  => format!("Color.const({})", rgba.lua_to_string()),
            DynColor::Custom(rgba) => format!("Color.custom({})", rgba.lua_to_string()),
            DynColor::Named(name)  => format!("Color.named(\"{}\")", name),
        }
    }
}
impl FromStr for DynColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "background" {
            Ok(DynColor::Background)
        } else if s.starts_with("#") || s.starts_with("linear(") {
            Ok(DynColor::Custom(RgbaColor::from_str(s)?))
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
impl RoughlyEq<DynColor> for &DynColor {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon { 0.0001 }

    fn roughly_eq_within(&self, that: &DynColor, epsilon: Self::Epsilon) -> bool {
        match (self, that) {
            (DynColor::Background, DynColor::Background) => true,
            (DynColor::Const(a),  DynColor::Const(b)) => a.roughly_eq_within(b, epsilon),
            (DynColor::Named(a),  DynColor::Named(b)) => a == b,
            (DynColor::Custom(a), DynColor::Custom(b)) => a.roughly_eq_within(b, epsilon),
            _ => false,
        }
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
        methods.add_method("eval", |lua, this, handle: Option<LuaHandle>| match handle {
            Some(handle) => {
                let handle = handle.try_palette()?;
                this.eval_lua(&handle, lua)
            },
            None         => this.eval_lua_current(lua),
        });
        // and we can call() as a synonym for evaling
        methods.add_meta_method(LuaMetaMethod::Call, |lua, this, handle: Option<LuaHandle>| match handle {
            Some(handle) => {
                let handle = handle.try_palette()?;
                this.eval_lua(&handle, lua)
            },
            None         => this.eval_lua_current(lua),
        });
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: DynColor| Ok(this.roughly_eq_within(&that, 0.0000001)));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(this.lua_to_string()));
    }
}
impl LuaMod for DynColor {
    fn mod_name() -> &'static str { "Color" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("background", DynColor::Background.to_lua(lua)?)?;
        table.set("const", lua.create_function(|_, rgba: RgbaColor| Ok(DynColor::Const(rgba)))?)?;
        table.set("custom", lua.create_function(|_, rgba: RgbaColor| Ok(DynColor::Custom(rgba)))?)?;
        table.set("named", lua.create_function(|_, name: String| {
            Ok(DynColor::Named(name))
        })?)?;
        table.set("transparent", lua.create_function(|_, ()| Ok(DynColor::Const(RgbaColor::TRANSPARENT)))?)?;

        let meta = table.get_metatable().unwrap_or(lua.create_table()?);
        meta.set(LuaMetaMethod::Call.name(), lua.create_function(|_, (_this, str): (LuaTable, String)| {
            DynColor::from_str(str.as_str())
                .map_err(|e| mlua::Error::RuntimeError(e))
        })?)?;
        table.set_metatable(Some(meta));
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
        params: Vec<ScriptVar>,
    },
    Const(RgbaColor),
}
impl ColorMiss {
    pub fn lua_to_string(&self, lua: &Lua) -> Result<String, mlua::Error> {
        match self {
            ColorMiss::Identity => Ok("\"identity\"".to_string()),
            ColorMiss::Clamp    => Ok("\"clamp\"".to_string()),
            ColorMiss::Const(c) => Ok(c.lua_to_string()),
            ColorMiss::Fn { file, function, params } => {
                let params_str = if params.is_empty() {
                    "".to_string()
                } else {
                    lua_to_string(params.clone().to_lua(lua)?)?
                };
                Ok(format!(
                    "{{file = {}, fn = {}{}}}",
                    file,
                    function,
                    params_str,
                ))
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaletteConfig {
    #[serde(default)]
    pub base:          Option<String>,
    #[serde(default)]
    pub on_miss:       ColorMiss,
    pub background:    DynColor,
    #[serde(default = "default_missing_rgba")]
    pub missing_rgba:  RgbaColor,
    pub colors:        HashMap<String, String>,
}
pub fn default_missing_rgba() -> RgbaColor { RgbaColor::FUCHSIA }

#[derive(Clone, Debug, PartialEq, TypeUuid)]
#[uuid = "f1c78ac4-576b-4504-85b6-96e5bf3bd9e1"]
pub struct Palette {
    pub handle:              Handle<Palette>, // self-reference; makes some ColorCache stuff easier
    pub on_miss:             ColorMiss,
    pub background:          DynColor,
    pub background_original: DynColor,
    pub missing_rgba:        RgbaColor,
    pub colors:              HashMap<String, RgbaColor>,
    pub colors_lch:          HashMap<String, Lcha>,      // colors in LCHA format, precomputed for clamping to palette
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

    pub fn clamp<C>(&self, c: C) -> C where C: FromColor<Lcha>, Lcha: FromColor<C> {
        let mut lch = Lcha::from_color(c);
        let mut dist = f32::MAX;
        for color in self.colors_lch.values() {
            let ndist = color.get_color_difference(&lch);
            if ndist < dist {
                dist = ndist;
                lch = *color;
            }
        }
        C::from_color(lch)
    }
}
impl Default for Palette {
    fn default() -> Self {
        DEFAULT_PALETTE.clone()
    }
}
impl<'de> Deserialize<'de> for Palette {
    fn deserialize<D>(d: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let mut colors = HashMap::<String, RgbaColor>::new();
        let mut color_refs = HashMap::new();
        let config: PaletteConfig = Deserialize::deserialize(d)?;
        if config.background == DynColor::Background {
            return Err(de::Error::custom("Background is defined recursively as \"background\""));
        }
        for (k, full) in config.colors {
            match full.chars().next().ok_or_else(|| de::Error::custom("Color string cannot be empty"))? {
                '#' => {
                    let c = Color::hex(&full[1..].to_string()).map_err(|e| de::Error::custom(e))?;
                    colors.insert(k, c.into());
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
        let colors_lch = colors.iter().map(|(k, v)| (k.clone(), Lcha::from_color(*v))).collect();
        Ok(Palette {
            handle: default(), // must be immediately set when to actual value palette is loaded
            colors,
            colors_lch,
            background: config.background.clone(),
            background_original: config.background,
            missing_rgba: config.missing_rgba,
            on_miss: config.on_miss.clone(),
        })
    }
}
impl LuaUserData for Palette {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("background", |_, this| Ok(this.background.clone()));
        fields.add_field_method_set("background", |_, this, any: Any3<DynColor, RgbaColor, String>| Ok(match any {
            Any3::A(c) => { this.background = c.clone() },
            Any3::B(c) => { this.background = DynColor::Custom(c) },
            Any3::C(c) => { this.background = DynColor::Named(c.clone()) },
        }));
        fields.add_field_method_get("missing_rgba", |_, this| Ok(this.background.clone()));
        fields.add_field_method_set("missing_rgba", |_, this, missing_rgba| Ok(this.missing_rgba = missing_rgba));
        fields.add_field_method_get("on_miss", |lua, this| match &this.on_miss {
            ColorMiss::Clamp       => "clamp".to_lua(lua),
            ColorMiss::Const(rgba) => rgba.to_lua(lua),
            ColorMiss::Identity    => "identity".to_lua(lua),
            ColorMiss::Fn { file, function, params } => {
                let table = lua.create_table()?;
                table.set("file", file.clone())?;
                table.set("fn", function.clone())?;
                if !params.is_empty() {
                    table.set("fn", params.clone())?;
                }
                table.to_lua(lua)
            },
        });
        fields.add_field_method_set("on_miss", |_, this, any: Any3<String, RgbaColor, LuaTable>| match any {
            Any3::A(s) => match s.as_str() {
                "clamp"    => Ok(this.on_miss = ColorMiss::Clamp),
                "identity" => Ok(this.on_miss = ColorMiss::Identity),
                _ => Err(mlua::Error::RuntimeError(format!("Unknown on_miss string \"{}\"", s))),
            },
            Any3::B(c) => Ok(this.on_miss = ColorMiss::Const(c)),
            Any3::C(table) => {
                let file     = table.get::<_, String>("file")?;
                let function = table.get::<_, String>("fn")?;
                let params   = table.get::<_, Option<Vec<ScriptVar>>>("params")?.unwrap_or(Vec::with_capacity(0));
                Ok(this.on_miss = ColorMiss::Fn { file, function, params })
            },
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("apply", |lua, this, handle: LuaHandle| {
            let handle = handle.try_palette()?;
            // must precache some values
            let mut palette = this.clone();
            palette.handle  = handle.clone_weak();
            palette.background_original = this.background.clone();

            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut palettes = w.resource_mut::<Assets<Palette>>();
            Ok(LuaHandle::from(palettes.set(handle, palette)))
        });
        methods.add_method_mut("clamp", |_, this, rgba: RgbaColor| {
            Ok(this.clamp(rgba))
        });
        methods.add_method("clone", |_, this, ()| Ok(this.clone()));
        methods.add_method("get", |_, this, name: String| Ok(this.colors.get(&name).cloned()));
        methods.add_method_mut("remove", |_, this, name: String| Ok({
            this.colors_lch.remove(&name);
            this.colors.remove(&name)
        }));
        methods.add_method_mut("set", |_, this, (name, rgba): (String, RgbaColor)| Ok({
            this.colors_lch.insert(name.clone(), Lcha::from_color(rgba));
            this.colors.insert(name, rgba)
        }));
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, that: Palette| Ok(this == &that));
        methods.add_meta_method(LuaMetaMethod::Index, |_, this, name: String| Ok(this.colors.get(&name).cloned()));
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, ()| Ok(this.colors.len()));
        methods.add_meta_method_mut(LuaMetaMethod::NewIndex, |_, this, (name, rgba): (String, RgbaColor)| {
            this.colors.insert(name.clone(), rgba);
            this.colors_lch.insert(name, Lcha::from_color(rgba));
            Ok(())
        });
        methods.add_meta_method(LuaMetaMethod::Pairs, |lua, this, ()| {
            let table = lua.create_table()?;
            for (k, v) in this.colors.iter() {
                table.set(k.clone(), *v)?;
            }
            let mut multi = LuaMultiValue::new();
            multi.push_front(LuaValue::Nil);
            multi.push_front(table.to_lua(lua)?);
            let next = lua.globals().get::<_, LuaFunction>("next")?;
            multi.push_front(next.to_lua(lua)?);
            Ok(multi)
        });
        methods.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| Ok(format!(
            "palette {{background = {}, on_miss = {}, missing_rgba = {}, colors = {{{}}}}}",
            this.background.lua_to_string(),
            this.on_miss.lua_to_string(lua)?,
            this.missing_rgba.lua_to_string(),
            this.colors.iter()
                .map(|(k, v)| format!("{} = {}", k, v.lua_to_string()))
                .intersperse(", ".to_string())
                .collect::<String>(),
        )));
    }
}
impl LuaMod for Palette {
    fn mod_name() -> &'static str { "Palette" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("add", lua.create_function(|lua, palette: Palette| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut palettes = w.resource_mut::<Assets<Palette>>();
            Ok(LuaHandle::from(palettes.add(palette)))
        })?)?;
        table.set("current", lua.create_function(|lua, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let loaded_palettes = w.resource::<LoadedPalettes>();
            Ok(LuaHandle::from(loaded_palettes.current_handle.clone()))
        })?)?;
        table.set("load", lua.create_function(|lua, path: String| {
            let path = fix_missing_extension::<PaletteLoader>(path);
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let asset_server = w.resource::<AssetServer>();
            let handle: Handle<Palette> = asset_server.load(&path);
            Ok(LuaHandle::from(handle))
        })?)?;
        table.set("new", lua.create_function(|_, ()| {
            Ok(Palette {
                handle:  default(),
                on_miss: default(),
                background: DynColor::CONST_BLACK,
                background_original: DynColor::CONST_BLACK,
                missing_rgba: RgbaColor::FUCHSIA,
                colors: HashMap::new(),
                colors_lch: HashMap::new(),
            })
        })?)?;
        table.set("swap", lua.create_function(|lua, handle: LuaHandle| {
            let handle = handle.handle.clone().typed();
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let last_handle = {
                let loaded_palettes = w.resource::<LoadedPalettes>();
                loaded_palettes.current_handle.clone()
            };
            w.insert_resource(LoadingPalette { handle });
            Ok(LuaHandle::from(last_handle))
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
    pub fn simple_rgba(
        &mut self,
        color:            &DynColor,
        loaded_palettes:  &LoadedPalettes,
        palettes:         &Assets<Palette>,
        shared_instances: &SharedInstances
    ) -> RgbaColor {
        if let Some(palette) = palettes.get(&loaded_palettes.current_handle) {
            if let Some(i) = loaded_palettes.lua_instance(shared_instances) {
                self.rgba(color, palette, i)
            } else {
                RgbaColor::FUCHSIA
            }
        } else { RgbaColor::FUCHSIA }
    }

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
                        ColorMiss::Clamp => palette.clamp(rgba),
                        ColorMiss::Const(rgba) => *rgba,
                        ColorMiss::Identity => rgba,
                        ColorMiss::Fn { function, params,.. } => { // uh oh, danger!
                            let mut new_params = Vec::new();
                            new_params.push(TransVar::Var(ScriptVar::Rgba(rgba)));
                            new_params.push(TransVar::Handle(LuaHandle::from(palette.handle.clone_weak())));
                            for param in params.iter() {
                                new_params.push(TransVar::Var(param.clone()));
                            }
                            lua_readable.with_read(|lua| {
                                let globals = lua.globals();
                                if let Some(f) = globals.get::<_, Option<LuaFunction>>(function.as_str()).unwrap() {
                                    match f.call::<_, RgbaColor>(ManyTransVars(new_params)) {
                                        Ok(c)    => c,
                                        Err(err) => {
                                            warn!("Function for on_miss Fn {} errored: {}", function, err);
                                            RgbaColor::FUCHSIA
                                        }
                                    }
                                } else {
                                    warn!("Function not found for on_miss Fn {}", function);
                                    RgbaColor::FUCHSIA
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
                        palette.missing_rgba
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