use std::{collections::{HashSet}, sync::atomic::{AtomicU64, Ordering}};

use bevy::{prelude::*, reflect::TypeUuid, asset::HandleId, app::AppExit};
use bevy_inspector_egui::egui::panel::Side;
use egui::panel::TopBottomSide;
use indexmap::IndexMap;
use mlua::prelude::*;

use crate::{data::{lua::{LuaWorld, Any3}, palette::DynColor}, scripting::{bevy_api::{math::LuaVec2, handle::LuaHandle}, LuaMod}};

use super::{atom::OrAtom, text::TextBuilder};

#[derive(Clone, Debug, Default, Resource)]
pub struct VisibleContainers(pub HashSet<HandleId>);


#[derive(Clone, Debug)]
pub struct TextInst {
    pub id:      u64,
    pub builder: TextBuilder,
}

#[derive(Debug)]
pub struct Elem {
    pub is_visible: OrAtom<bool>,
    pub kind:       ElemKind,
    pub on_click:   Option<LuaRegistryKey>,
    pub size:       OrAtom<Option<Vec2>>,
    pub tooltip:    Option<OrAtom<TextInst>>,
}

#[derive(Debug)]
pub enum ElemKind {
    Horizontal {
        children: Vec<Elem>,
    },
    Vertical {
        children: Vec<Elem>,
    },
    Button {
        text: OrAtom<TextInst>,
    },
    ImageButton {
        color:     OrAtom<DynColor>,
        is_framed: OrAtom<bool>,
        image:     OrAtom<Handle<Image>>,
    },
    Label {
        text: OrAtom<TextInst>,
    },
    Menu {
        children: IndexMap<OrAtom<String>, Vec<Elem>>,
    },
    SidePanel {
        name:     String,
        side:     Side,
        children: Vec<Elem>,
    },
    VerticalPanel {
        name:     String,
        side:     TopBottomSide,
        children: Vec<Elem>,
    },
}

#[derive(Debug, TypeUuid)]
#[uuid = "e1ebd6b3-14ca-43c9-a35f-39267f3f6ba6"]
pub struct Container {
    pub script_id: u32,
    pub elems: Vec<Elem>,
}

static NEXT_ELEM_ID: AtomicU64 = AtomicU64::new(0);

fn process_elem(lua: &Lua, table: LuaTable) -> Result<Elem, mlua::Error> {
    fn process_children(lua: &Lua, table: LuaTable) -> Result<Vec<Elem>, mlua::Error> {
        let mut children: Vec<Elem> = Vec::new();
        for r in table.pairs::<LuaValue, LuaValue>() {
            let (k, v) = r?;
            match k {
                // numeric key, assume these are from the array portion of table
                LuaValue::Integer(_) | LuaValue::Number(_) => {
                    if let LuaValue::Table(sub_table) = v {
                        children.push(process_elem(lua, sub_table)?);
                    } else {
                        warn!("expected table, got {:?}", v);
                    }
                },
                _ => (),
            }
        }
        Ok(children)
    }

    let size = table.get::<_, Option<OrAtom<Option<LuaVec2>>>>("size")?
        .map(|atom| atom.map(|o| o.map(|v| v.0)))
        .unwrap_or(OrAtom::Val(None));

    let is_visible = table.get::<_, Option<OrAtom<bool>>>("is_visible")?.unwrap_or(OrAtom::Val(true));

    let on_click: Option<LuaRegistryKey> = if let Some(f) = table.get::<_, Option<LuaFunction>>("on_click")? {
        Some(lua.create_registry_value(f)?)
    } else { None };

    let tooltip = table.get::<_, Option<OrAtom<TextBuilder>>>("tooltip")?.map(|t| t.map(|builder| TextInst {
        builder,
        id: NEXT_ELEM_ID.fetch_add(1, Ordering::Acquire),
    }));

    let kind = if let Some(k) = table.get::<_, Option<String>>("kind")? {
        match k.as_str() {
            "horizontal"  => {
                let children = process_children(lua, table)?;
                ElemKind::Horizontal { children }
            },
            "vertical"  => {
                let children = process_children(lua, table)?;
                ElemKind::Vertical { children }
            },
            "button" => {
                let text = table.get::<_, OrAtom<TextBuilder>>("text")?.map(|builder| TextInst {
                    builder,
                    id: NEXT_ELEM_ID.fetch_add(1, Ordering::Acquire),
                });
                ElemKind::Button { text }
            },
            "imagebutton" => {
                let image = match table.get::<_, OrAtom<LuaHandle>>("image")? {
                    OrAtom::Val(i)  => OrAtom::Val(i.try_image()?),
                    OrAtom::Atom(r) => OrAtom::Atom(r),
                };
                let color = table.get::<_, OrAtom<DynColor>>("color")?;
                let is_framed = table.get::<_, OrAtom<bool>>("is_framed")?;
                ElemKind::ImageButton { image, is_framed, color }
            },
            "label" => {
                let text = table.get::<_, OrAtom<TextBuilder>>("text")?.map(|builder| TextInst {
                    builder,
                    id: NEXT_ELEM_ID.fetch_add(1, Ordering::Acquire),
                });
                ElemKind::Label { text }
            },
            "menu" => {
                let mut last_cat: Option<OrAtom<String>> = None;
                let mut children = IndexMap::new();
                for r in table.pairs::<LuaValue, LuaValue>() {
                    let (k, v) = r?;
                    match k {
                        // numeric key, assume these are from the array portion of table
                        LuaValue::Integer(_) | LuaValue::Number(_) => {
                            if let LuaValue::Table(sub_table) = v {
                                let name = sub_table.get::<_, Option<OrAtom<String>>>("menubar")?.or_else(|| last_cat.clone()).ok_or_else(|| mlua::Error::RuntimeError(
                                    format!("first menu child had no menubar; all children of menus must either have a menubar, else they inherit the last ones"),
                                ))?;
                                last_cat = Some(name.clone());
                                let vec = children.entry(name).or_insert(Vec::new());
                                vec.push(process_elem(lua, sub_table)?);
                            } else {
                                warn!("expected table, got {:?}", v);
                            }
                        },
                        _ => (),
                    }
                }
                ElemKind::Menu { children }
            },
            "sidepanel" => {
                let side = if let Some(side) = table.get::<_, Option<String>>("anchor")? {
                    match side.as_str() {
                        "left" => Side::Left,
                        "right" => Side::Right,
                        _ => {
                            return Err(mlua::Error::RuntimeError(format!("Expected side = \"left\" or \"right\", found {}", side)));
                        },
                    }
                } else { Side::Left };
                let name = format!("__sp#{}", NEXT_ELEM_ID.fetch_add(1, Ordering::Acquire));
                let children = process_children(lua, table)?;
                ElemKind::SidePanel { name, side, children }
            },
            "verticalpanel" => {
                let side = if let Some(side) = table.get::<_, Option<String>>("anchor")? {
                    match side.as_str() {
                        "top" => TopBottomSide::Top,
                        "bottom" => TopBottomSide::Bottom,
                        _ => {
                            return Err(mlua::Error::RuntimeError(format!("Expected side = \"top\" or \"bottom\", found {}", side)));
                        },
                    }
                } else { TopBottomSide::Top };
                let name = format!("__vp#{}", NEXT_ELEM_ID.fetch_add(1, Ordering::Acquire));
                let children = process_children(lua, table)?;
                ElemKind::VerticalPanel { name, side, children }
            },
            _ => {
                return Err(mlua::Error::RuntimeError(format!("unknown elem table kind {}", k)));
            },
        }
    } else {
        return Err(mlua::Error::RuntimeError(format!("elem table has no kind")));
    };
    Ok(Elem { is_visible, kind, on_click, size, tooltip })
}

#[derive(Default)]
pub struct UIAPI;
impl LuaMod for UIAPI {
    fn mod_name() -> &'static str { "UI" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("compile", lua.create_function(|lua, multi: LuaMultiValue| {
            let mut container = Container {
                script_id: lua.globals().get::<_, u32>("script_id").unwrap(),
                elems: Vec::new(),
            };
            for r in multi {
                let v: LuaTable = LuaTable::from_lua(r, lua)?;
                let elem = process_elem(lua, v)?;
                container.elems.push(elem);
            }
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut containers = w.resource_mut::<Assets<Container>>();
            Ok(LuaHandle::from(containers.add(container)))
        })?)?;
        table.set("horizontal", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "horizontal")?;
            Ok(table)
        })?)?;
        table.set("vertical", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "vertical")?;
            Ok(table)
        })?)?;
        table.set("button", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "button")?;
            Ok(table)
        })?)?;
        table.set("imagebutton", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "imagebutton")?;
            Ok(table)
        })?)?;
        table.set("label", lua.create_function(|lua, any: Any3::<OrAtom<TextBuilder>, LuaTable, String>| {
            let table = match any {
                Any3::A(atom) => {
                    // TextBuilder wrapped in a table
                    let table = lua.create_table()?;
                    match atom {
                        OrAtom::Atom(r) => table.set("text", r)?,
                        OrAtom::Val(t)  => table.set("text", t)?,
                    }
                    table
                },
                Any3::B(table) => table,
                Any3::C(string) => {
                    let table = lua.create_table()?;
                    table.set("text", TextBuilder::plain(string))?;
                    table
                },
            };
            table.set("kind", "label")?;
            Ok(table)
        })?)?;
        table.set("menu", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "menu")?;
            Ok(table)
        })?)?;
        table.set("sidepanel", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "sidepanel")?;
            Ok(table)
        })?)?;
        table.set("verticalpanel", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "verticalpanel")?;
            Ok(table)
        })?)?;

        table.set("queue_app_exit", lua.create_function(|lua, ()| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            w.send_event(AppExit);
            Ok(())
        })?)?;
        table.set("hide", lua.create_function(|lua, handle: LuaHandle| {
            let handle = handle.try_ui_container()?;
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut visibilities = w.resource_mut::<VisibleContainers>();
            visibilities.0.remove(&handle.id());
            Ok(())
        })?)?;
        table.set("show", lua.create_function(|lua, handle: LuaHandle| {
            let handle = handle.try_ui_container()?;
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let mut w = world.write();
            let mut visibilities = w.resource_mut::<VisibleContainers>();
            visibilities.0.insert(handle.id());
            Ok(())
        })?)?;

        Ok(())
    }
}