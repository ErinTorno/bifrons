use std::{collections::HashSet, sync::atomic::{AtomicU64, Ordering}};

use bevy::{prelude::*, reflect::TypeUuid, asset::HandleId};
use bevy_inspector_egui::egui::panel::Side;
use mlua::prelude::*;

use crate::{data::{lua::{LuaWorld}, palette::DynColor}, scripting::{bevy_api::{math::LuaVec2, handle::LuaHandle}, LuaMod}};

use super::atom::OrAtom;

#[derive(Clone, Debug, Default, Resource)]
pub struct VisibleContainers(pub HashSet<HandleId>);

#[derive(Debug)]
pub struct Elem {
    pub is_visible: OrAtom<bool>,
    pub kind:       ElemKind,
    pub size:       OrAtom<Option<Vec2>>,
}

#[derive(Debug)]
pub enum ElemKind {
    Horizontal {
        children: Vec<Elem>,
    },
    Vertical {
        children: Vec<Elem>,
    },
    ImageButton {
        color:     OrAtom<DynColor>,
        on_click:  Option<LuaRegistryKey>,
        is_framed: OrAtom<bool>,
        image:     OrAtom<Handle<Image>>,
    },
    SidePanel {
        name:     String,
        side:     Side,
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
    let mut children: Vec<Elem> = Vec::new();
    for r in table.clone().pairs::<LuaValue, LuaValue>() {
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

    let size = table.get::<_, Option<OrAtom<Option<LuaVec2>>>>("size")?
        .map(|atom| atom.map(|o| o.map(|v| v.0)))
        .unwrap_or(OrAtom::Val(None));

    let is_visible = table.get::<_, Option<OrAtom<bool>>>("is_visible")?.unwrap_or(OrAtom::Val(true));

    fn warn_no_children_expected(children: &Vec<Elem>, kind: &str) {
        if !children.is_empty() {
            warn!("No children expected for ui elem of kind {}", kind);
        }
    }

    let kind = if let Some(k) = table.get::<_, Option<String>>("kind")? {
        match k.as_str() {
            "horizontal"  => {
                ElemKind::Horizontal { children }
            },
            "vertical"  => {
                ElemKind::Vertical { children }
            },
            "imagebutton" => {
                warn_no_children_expected(&children, "imagebutton");
                let on_click: Option<LuaRegistryKey> = if let Some(f) = table.get::<_, Option<LuaFunction>>("on_click")? {
                    Some(lua.create_registry_value(f)?)
                } else { None };
                let image = match table.get::<_, OrAtom<LuaHandle>>("image")? {
                    OrAtom::Val(i)  => OrAtom::Val(i.try_image()?),
                    OrAtom::Atom(r) => OrAtom::Atom(r),
                };
                let color = table.get::<_, OrAtom<DynColor>>("color")?;
                let is_framed = table.get::<_, OrAtom<bool>>("is_framed")?;
                ElemKind::ImageButton { on_click, image, is_framed, color }
            },
            "sidepanel" => {
                let side = if let Some(side) = table.get::<_, Option<String>>("anchor")? {
                    match side.as_str() {
                        "left" => Side::Left,
                        "right" => Side::Right,
                        _ => {
                            return Err(mlua::Error::RuntimeError(format!("Expected side = 'left' or 'right', found {}", side)));
                        },
                    }
                } else { Side::Left };
                let name = format!("sidepanel#{}", NEXT_ELEM_ID.fetch_add(1, Ordering::Acquire));
                ElemKind::SidePanel { name, side, children }
            },
            _ => {
                return Err(mlua::Error::RuntimeError(format!("unknown elem table kind {}", k)));
            },
        }

    } else {
        return Err(mlua::Error::RuntimeError(format!("elem table has no kind")));
    };
    Ok(Elem { is_visible, kind, size })
}

#[derive(Default)]
pub struct UIAPI;
impl LuaMod for UIAPI {
    fn mod_name() -> &'static str { "UI" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("add", lua.create_function(|lua, table: LuaTable| {
            let mut container = Container {
                script_id: lua.globals().get::<_, u32>("script_id").unwrap(),
                elems: Vec::new(),
            };
            for r in table.pairs::<LuaValue, LuaTable>() {
                let (_, v) = r?;
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
        table.set("imagebutton", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "imagebutton")?;
            Ok(table)
        })?)?;
        table.set("sidepanel", lua.create_function(|_, table: LuaTable| {
            table.set("kind", "sidepanel")?;
            Ok(table)
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