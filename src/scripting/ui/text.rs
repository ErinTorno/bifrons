use std::{default::default, hash::{Hash, Hasher}};

use bevy::prelude::{Handle};
use bevy_egui::egui;
use egui::{text::LayoutJob, FontFamily, FontId, TextFormat};
use lazy_static::lazy_static;
use mlua::prelude::*;

use crate::{scripting::{LuaMod, color::RgbaColor, bevy_api::handle::LuaHandle}, data::{palette::{DynColor, LuaToDynColor}, lua::{Any3, Any2, LuaWorld}}, system::ui::UIAssets, util::RoughToBits};

use super::font::UIFont;

#[derive(Clone, Debug, PartialEq)]
pub struct DynStroke {
    pub width: f32,
    pub color: DynColor,
}
impl Hash for DynStroke {
    fn hash<H>(&self, hasher: &mut H) where H: Hasher {
        hasher.write_u32(self.width.rough_to_bits());
        self.color.hash(hasher);
    }
}
impl DynStroke {
    pub const NONE: DynStroke = DynStroke {
        width: 0.,
        color: DynColor::CONST_TRANSPARENT,
    };

    pub fn eval<F>(&self, f: F) -> egui::Stroke where F: FnOnce(&DynColor) -> RgbaColor {
        egui::Stroke {
            width: self.width,
            color: f(&self.color).into(),
        }
    }

    pub fn from_table(table: &LuaTable) -> Result<DynStroke, mlua::Error> {
        let width = table.get::<_, Option<_>>("width")?.unwrap_or(0.);
        let color = if let Some(any) = table.get::<_, Option<_>>("color")? {
            match any {
                Any3::A(c) => c,
                Any3::B(c) => DynColor::Custom(c),
                Any3::C(c) => DynColor::Named(c),
            }
        } else { DynColor::Named("white".to_string()) };
        Ok(DynStroke { width, color })
    }

    pub fn to_table<'a>(&self, lua: &'a Lua) -> Result<LuaTable<'a>, mlua::Error> {
        let table = lua.create_table()?;
        table.set("color", self.color.clone())?;
        table.set("width", self.width)?;
        Ok(table)
    }
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct TextStyle {
    pub background:    DynColor,
    pub color:         DynColor,
    pub font:          FontId,
    pub font_id:       Option<Handle<UIFont>>,
    pub italics:       bool,
    pub underline:     DynStroke,
    pub strikethrough: DynStroke,
    pub valign:        egui::Align,
}
lazy_static! {
    pub static ref DEFAULT_TEXT_STYLE: TextStyle = TextStyle {
        background:    DynColor::CONST_TRANSPARENT,
        color:         DynColor::Named("white".to_string()),
        font:          FontId::default(),
        font_id:       None,
        italics:       false,
        strikethrough: DynStroke::NONE,
        underline:     DynStroke::NONE,
        valign:        egui::Align::Center,
    };
}
impl TextStyle {
    pub fn to_text_format<F>(self, mut eval_color: F) -> TextFormat where F: FnMut(&DynColor) -> RgbaColor {
        TextFormat {
            font_id:       self.font,
            color:         eval_color(&self.color).into(),
            background:    eval_color(&self.background).into(),
            italics:       self.italics,
            underline:     self.underline.eval(|c| eval_color(c)),
            strikethrough: self.strikethrough.eval(|c| eval_color(c)),
            valign:        self.valign,
        }
    }

    pub fn font_from_any(lua: &Lua, any: Any2<LuaHandle, String>) -> Result<(FontFamily, Option<Handle<UIFont>>), mlua::Error> {
        Ok(match any {
            Any2::A(handle) => {
                let handle = handle.try_font()?;
                let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                let w = world.read();
                let ui_assets = w.resource::<UIAssets>();
                let name = ui_assets.names_by_font.get(&handle)
                    .ok_or_else(|| mlua::Error::RuntimeError(format!("No known name found for Handle<Font> {:?}", handle)))?;
                (FontFamily::Name(name.as_str().into()), Some(handle))
            },
            Any2::B(str) => {
                (match str.as_str() {
                    "monospace"    => FontFamily::Monospace,
                    "proportional" => FontFamily::Proportional,
                    _ => {
                        return Err(mlua::Error::RuntimeError(format!("Unknown font \"{}\"; valid vals: Handle<Font>, \"monospace\", \"proportional\"", str)));
                    },
                }, None)
            },
        })
    }

    pub fn set_font_family(&mut self, lua: &Lua, any: Any2<LuaHandle, String>) -> Result<(), mlua::Error> {
        let (font_family, font_id) = TextStyle::font_from_any(lua, any)?;
        self.font_id = font_id;
        Ok(self.font.family = font_family)
    }
}
impl Default for TextStyle {
    fn default() -> Self { DEFAULT_TEXT_STYLE.clone() }
}
impl LuaUserData for TextStyle {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("background", |_, this| Ok(this.color.clone()));
        fields.add_field_method_set("background", |_, this, any: Any3<DynColor, RgbaColor, String>| Ok(match any {
            Any3::A(c) => { this.background = c },
            Any3::B(c) => { this.background = DynColor::Custom(c) },
            Any3::C(c) => { this.background = DynColor::Named(c) },
        }));
        fields.add_field_method_get("color", |_, this| Ok(this.color.clone()));
        fields.add_field_method_set("color", |_, this, any: Any3<DynColor, RgbaColor, String>| Ok(match any {
            Any3::A(c) => { this.color = c },
            Any3::B(c) => { this.color = DynColor::Custom(c) },
            Any3::C(c) => { this.color = DynColor::Named(c) },
        }));
        fields.add_field_method_get("font", |lua, this| match this.font.family {
            FontFamily::Monospace    => "monospace".to_lua(lua),
            FontFamily::Proportional => "proportional".to_lua(lua),
            _ => this.font_id.clone().map(|f| LuaHandle::from(f)).to_lua(lua),
        });
        fields.add_field_method_set("font", |lua, this, any| this.set_font_family(lua, any));
        fields.add_field_method_get("font_size", |_, this| Ok(this.font.size));
        fields.add_field_method_set("font_size", |_, this, size| Ok(this.font.size = size));
        fields.add_field_method_get("italics", |_, this| Ok(this.italics));
        fields.add_field_method_set("italics", |_, this, italics| Ok(this.italics = italics));
        fields.add_field_method_get("strikethrough", |lua, this| this.strikethrough.to_table(lua));
        fields.add_field_method_set("strikethrough", |_, this, table| Ok(this.strikethrough = DynStroke::from_table(&table)?));
        fields.add_field_method_get("underline", |lua, this| this.underline.to_table(lua));
        fields.add_field_method_set("underline", |_, this, table| Ok(this.underline = DynStroke::from_table(&table)?));
        fields.add_field_method_get("valign", |_, this| Ok(match this.valign {
            egui::Align::Center => "center",
            egui::Align::BOTTOM => "top",
            egui::Align::TOP    => "bottom",
        }.to_string()));
        fields.add_field_method_set("valign", |_, this, align: String| Ok(this.valign = match align.as_str() {
            "bottom" => egui::Align::BOTTOM,
            "center" => egui::Align::Center,
            "top"    => egui::Align::TOP,
            _ => {
                return Err(mlua::Error::RuntimeError(format!("Unknown valign {}; expected \"bottom\", \"center\", \"top\"", align)));
            },
        }));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("clone", |_, this, ()| Ok(this.clone()));

        methods.add_function_mut("with_background", |_, (this, any): (LuaAnyUserData, Any3<DynColor, RgbaColor, String>)| {
            let mut this: TextStyle = this.take()?;
            match any {
                Any3::A(c) => { this.background = c },
                Any3::B(c) => { this.background = DynColor::Custom(c) },
                Any3::C(c) => { this.background = DynColor::Named(c) },
            }
            Ok(this)
        });
        methods.add_function_mut("with_color", |_, (this, any): (LuaAnyUserData, Any3<DynColor, RgbaColor, String>)| {
            let mut this: TextStyle = this.take()?;
            match any {
                Any3::A(c) => { this.color = c },
                Any3::B(c) => { this.color = DynColor::Custom(c) },
                Any3::C(c) => { this.color = DynColor::Named(c) },
            }
            Ok(this)
        });
        methods.add_function_mut("with_font", |lua, (this, any): (LuaAnyUserData, _)| {
            let mut this: TextStyle = this.take()?;
            this.set_font_family(lua, any)?;
            Ok(this)
        });
        methods.add_function_mut("with_font_size", |_, (this, font_size): (LuaAnyUserData, _)| {
            let mut this: TextStyle = this.take()?;
            this.font.size = font_size;
            Ok(this)
        });
        methods.add_function_mut("with_italics", |_, (this, italics): (LuaAnyUserData, _)| {
            let mut this: TextStyle = this.take()?;
            this.italics = italics;
            Ok(this)
        });
        methods.add_function_mut("with_strikethrough", |_, (this, table): (LuaAnyUserData, _)| {
            let mut this: TextStyle = this.take()?;
            this.strikethrough = DynStroke::from_table(&table)?;
            Ok(this)
        });
        methods.add_function_mut("with_underline", |_, (this, table): (LuaAnyUserData, _)| {
            let mut this: TextStyle = this.take()?;
            this.underline = DynStroke::from_table(&table)?;
            Ok(this)
        });
        methods.add_function_mut("with_valign", |_, (this, align): (LuaAnyUserData, String)| {
            let mut this: TextStyle = this.take()?;
            this.valign = match align.as_str() {
                "bottom" => egui::Align::Max,
                "center" => egui::Align::Center,
                "top"    => egui::Align::Min,
                _ => {
                    return Err(mlua::Error::RuntimeError(format!("Unknown valign {}; expected \"bottom\", \"center\", \"top\"", align)));
                },
            };
            Ok(this)
        });
    }
}
impl LuaMod for TextStyle {
    fn mod_name() -> &'static str { "TextStyle" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("from", lua.create_function(|lua, table: LuaTable| {
            let background = table.get::<_, Option<LuaToDynColor>>("background")?
                .map(DynColor::from_any)
                .unwrap_or_else(|| DEFAULT_TEXT_STYLE.background.clone());
            let color = table.get::<_, Option<LuaToDynColor>>("color")?
                .map(DynColor::from_any)
                .unwrap_or_else(|| DEFAULT_TEXT_STYLE.background.clone());

            let font_size = table.get::<_, Option<f32>>("font_size")?
                .unwrap_or(DEFAULT_TEXT_STYLE.font.size);
            let (font_family, font_id) = if let Some(any) = table.get("font")? {
                TextStyle::font_from_any(lua, any)?
            } else { (DEFAULT_TEXT_STYLE.font.family.clone(), DEFAULT_TEXT_STYLE.font_id.clone()) };
            let font = FontId::new(font_size, font_family);

            let italics = table.get::<_, Option<bool>>("italics")?
                .unwrap_or(DEFAULT_TEXT_STYLE.italics);

            let strikethrough = if let Some(table) = table.get("strikethrough")? {
                DynStroke::from_table(&table)?
            } else { DEFAULT_TEXT_STYLE.strikethrough.clone() };
            let underline = if let Some(table) = table.get("underline")? {
                DynStroke::from_table(&table)?
            } else { DEFAULT_TEXT_STYLE.underline.clone() };

            let valign = if let Some(align) = table.get::<_, Option<String>>("valign")? {
                match align.as_str() {
                    "bottom" => egui::Align::Max,
                    "center" => egui::Align::Center,
                    "top"    => egui::Align::Min,
                    _ => {
                        return Err(mlua::Error::RuntimeError(format!("Unknown valign {}; expected \"bottom\", \"center\", \"top\"", align)));
                    },
                }
            } else { DEFAULT_TEXT_STYLE.valign };

            Ok(TextStyle { background, color, font, font_id, italics, underline, strikethrough, valign })
        })?)?;
        table.set("new", lua.create_function(|_, ()| Ok(TextStyle::default()))?)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextSection {
    pub style:         TextStyle,
    pub leading_space: f32,
    pub text:          String,
}
impl Hash for TextSection {
    fn hash<H>(&self, hasher: &mut H) where H: Hasher {
        hasher.write_u32(self.leading_space.rough_to_bits());
        self.style.hash(hasher);
        self.text.hash(hasher);
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq)]
pub struct TextBuilder {
    pub sections: Vec<TextSection>,
    pub current:  TextSection,
}
impl TextBuilder {
    pub fn plain(text: String) -> TextBuilder {
        TextBuilder { sections: Vec::new(), current: TextSection { text, ..default() } }
    }

    pub fn to_layout_job<F>(self, mut eval_color: F) -> LayoutJob where F: FnMut(&DynColor) -> RgbaColor {
        let mut job = LayoutJob::default();

        for section in self.sections {
            job.append(&section.text, section.leading_space, section.style.to_text_format(|c| eval_color(c)));
        }
        if !self.current.text.is_empty() {
            let section = self.current;
            job.append(&section.text, section.leading_space, section.style.to_text_format(|c| eval_color(c)));
        }

        job
    }
}
impl LuaUserData for TextBuilder {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {
        
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("clone", |_, this, ()| Ok(this.clone()));

        methods.add_function_mut("push", |_, (this, str): (LuaAnyUserData, String)| {
            let mut this: TextBuilder = this.take()?;
            this.current.text.push_str(str.as_str());
            if this.current.leading_space >= 0. {
                this.sections.push(this.current.clone());
                this.current.leading_space = 0.;
                this.current.text = String::new();
            }
            Ok(this)
        });
        methods.add_function_mut("indent", |_, (this, indent): (LuaAnyUserData, f32)| {
            let mut this: TextBuilder = this.take()?;
            if !this.current.text.is_empty() {
                this.sections.push(this.current.clone());
                this.current.text = String::new();
            }
            this.current.leading_space += indent;
            Ok(this)
        });
        methods.add_function_mut("style", |_, (this, style): (LuaAnyUserData, _)| {
            let mut this: TextBuilder = this.take()?;
            
            if !this.current.text.is_empty() {
                this.sections.push(this.current);
                this.current = TextSection { style, ..default() };
            } else {
                this.current.style = style;
            }
            Ok(this)
        });
    }
}
impl LuaMod for TextBuilder {
    fn mod_name() -> &'static str { "Text" }

    fn register_defs(lua: &mlua::Lua, table: &mut mlua::Table) -> Result<(), mlua::Error> {
        table.set("new", lua.create_function(|_, any: Option<Any2<TextStyle, String>>| Ok(match any {
            None      => TextBuilder::default(),
            Some(any) => match any {
                Any2::A(style) => TextBuilder {
                    sections: Vec::new(),
                    current:  TextSection { style, leading_space: 0., text: String::new() },
                },
                Any2::B(str) => TextBuilder::plain(str),
            },
        }))?)?;
        Ok(())
    }
}