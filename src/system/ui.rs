use std::collections::{HashMap};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use mlua::prelude::*;

use crate::{scripting::{ui::{elem::*, atom::{LuaAtomRegistry, OrAtom}, font::UIFont}}, data::{lua::InstanceRef, palette::{Palette, ColorCache, LoadedPalettes, DynColor}}};

use super::lua::SharedInstances;

#[derive(Clone, Debug, Default)]
pub struct ScriptingUiPlugin;

impl Plugin for ScriptingUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .init_resource::<UIAssets>()
            .init_resource::<UIStateCache>()
            .init_resource::<VisibleContainers>()
            .add_startup_system(setup_default_ui_assets)
            .add_system(load_fonts)
            .add_system(run_containers)
        ;
    }
}

#[derive(Clone, Default, Resource)]
pub struct UIAssets {
    pub font_monospace:     Handle<UIFont>,
    pub font_proportional:  Handle<UIFont>,
    pub names_by_font:      HashMap<Handle<UIFont>, String>,
    pub texture_ids:        HashMap<Handle<Image>, egui::TextureId>,
    pub missing_texture:    Handle<Image>,
    pub missing_texture_id: egui::TextureId,
}
impl UIAssets {
    pub fn texture_id_or_insert(&mut self, h: Handle<Image>, egui_ctx: &mut EguiContext) -> egui::TextureId {
        self.texture_ids.entry(h.clone_weak()).or_insert_with(|| {
            egui_ctx.add_image(h.clone_weak())
        }).clone()
    }

    pub fn texture_or_def(&self, h: &Handle<Image>) -> egui::TextureId {
        match self.texture_ids.get(h) {
            Some(tex_id) => tex_id.clone(),
            None         => self.missing_texture_id.clone(),
        }
    }
}

#[derive(Clone, Default, Resource)]
pub struct UIStateCache {
    pub elem_is_open: HashMap<egui::Id, bool>,
}

pub fn is_ui_focused(mut egui_ctx: ResMut<EguiContext>) -> bool {
    let ctx = egui_ctx.ctx_mut();
    ctx.is_using_pointer() || ctx.is_pointer_over_area()
}

pub fn setup_default_ui_assets(
    asset_server:  Res<AssetServer>,
    mut ui_assets: ResMut<UIAssets>,
    mut egui_ctx:  ResMut<EguiContext>,
) {
    ui_assets.font_monospace    = asset_server.load("fonts/UbuntuMono-R.ttf");
    ui_assets.font_proportional = asset_server.load("fonts/Ubuntu-C.ttf");
    ui_assets.missing_texture = asset_server.load("missing.ktx2");
    let missing_handle = ui_assets.missing_texture.clone_weak();
    ui_assets.missing_texture_id = ui_assets.texture_id_or_insert(missing_handle, egui_ctx.as_mut());
}

pub fn load_fonts(
    fonts:         Res<Assets<UIFont>>,
    mut ui_assets: ResMut<UIAssets>,
    mut egui_ctx:  ResMut<EguiContext>,
    mut events:    EventReader<AssetEvent<UIFont>>,
) {
    for e in events.iter() {
        match e {
            AssetEvent::Created { handle: this_handle } => {
                let this_handle_id = this_handle.clone_weak_untyped().id;
                // todo find way that doesn't require copying all font data every time
                let mut font_defs = egui::FontDefinitions::default();

                let mono  = ui_assets.font_monospace.clone_weak_untyped().id;
                let propo = ui_assets.font_proportional.clone_weak_untyped().id;
                for (handle, font) in fonts.iter() {
                    font_defs.font_data.insert(font.name.clone(), font.data.clone());

                    if handle == mono {
                        font_defs.families.entry(egui::FontFamily::Monospace).or_default().insert(0, font.name.clone());
                    } else if handle == propo {
                        font_defs.families.entry(egui::FontFamily::Proportional).or_default().insert(0, font.name.clone());
                    }
                    if handle == this_handle_id {
                        ui_assets.names_by_font.insert(this_handle.clone_weak(), font.name.clone());
                    }
                }

                let ctx = egui_ctx.ctx_mut();
                ctx.set_fonts(font_defs);
            },
            _ => (),
        }
    }
}

enum ShowTo<'a> { TopLevel, Ui(&'a mut egui::Ui) }

pub fn run_containers(
    containers:       Res<Assets<Container>>,
    palettes:         Res<Assets<Palette>>,
    loaded_palettes:  Res<LoadedPalettes>,
    shared_instances: Res<SharedInstances>,
    ui_assets:        Res<UIAssets>,
    visibilities:     Res<VisibleContainers>,
    mut color_cache:  ResMut<ColorCache>,
    mut egui_ctx:     ResMut<EguiContext>,
    mut atom_reg:     ResMut<LuaAtomRegistry>,
    mut ui_cache:     ResMut<UIStateCache>,
) {
    fn call_on_click(maybe_key: &Option<LuaRegistryKey>, inst_ref: &InstanceRef) {
        if let Some(on_click) = maybe_key {
            let lua = inst_ref.lock.write();
            let f = lua.registry_value::<LuaFunction>(on_click).unwrap();
            if let Err(e) = f.call::<_, ()>(()) {
                warn!("ImageButton on_click error: {}", e);
            }
        }
    }
    struct RunElemEnv<'a> {
        inst_ref:    &'a InstanceRef,
        ctx:         &'a egui::Context,
        atom_reg:    &'a mut LuaAtomRegistry,
        color_cache: &'a mut ColorCache,
        ui_cache:    &'a mut UIStateCache,
    }
    struct RunElem<'a> { f: &'a dyn Fn(&RunElem, &Elem, &mut RunElemEnv, ShowTo) -> () }
    let run_elem = RunElem {
        f: &|run_elem, elem, env, show_to| {
            if env.atom_reg.acknowledge_or_else(elem.is_visible, || true) {
                let size = env.atom_reg.acknowledge_option(elem.size.clone());
                let size = size.map(|s| egui::Vec2::new(s.x, s.y));
                let response: Option<egui::Response> = match &elem.kind {
                    ElemKind::Horizontal { children } => {
                        match show_to {
                            ShowTo::TopLevel => { warn!("Horizontal not supported at TopLevel"); None },
                            ShowTo::Ui(ui) => {
                                Some(ui.horizontal(|ui| {
                                    for child in children.iter() {
                                        (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                    }
                                }).response)
                            },
                        }
                    },
                    ElemKind::Vertical { children } => {
                        match show_to {
                            ShowTo::TopLevel => { warn!("Vertical not supported at TopLevel"); None },
                            ShowTo::Ui(ui) => {
                                Some(ui.vertical(|ui| {
                                    for child in children.iter() {
                                        (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                    }
                                }).response)
                            },
                        }
                    },
                    ElemKind::Button { text } => {
                        match show_to {
                            ShowTo::TopLevel => { warn!("Button not supported at TopLevel"); None },
                            ShowTo::Ui(ui) => {
                                Some(ui.add(egui::Button::new(env.atom_reg.acknowledge_layout_job(
                                    &text,
                                    |c| env.color_cache
                                        .simple_rgba(&c, loaded_palettes.as_ref(), palettes.as_ref(), shared_instances.as_ref()),
                                    || "ERROR".to_string()
                                ))))
                            },
                        }
                    },
                    ElemKind::Hyperlink { url, text } => {
                        match show_to {
                            ShowTo::TopLevel => { warn!("Hyperlink not supported at TopLevel"); None },
                            ShowTo::Ui(ui) => {
                                let url  = env.atom_reg.acknowledge_or_else(url.clone(), || "".to_string());
                                Some(match text {
                                    Some(t) => ui.hyperlink_to(env.atom_reg.acknowledge_layout_job(
                                        &t,
                                        |c| env.color_cache
                                            .simple_rgba(&c, loaded_palettes.as_ref(), palettes.as_ref(), shared_instances.as_ref()),
                                        || "ERROR".to_string()
                                    ), url),
                                    None => ui.hyperlink(url),
                                })
                            },
                        }
                    },
                    ElemKind::ImageButton { image, color, is_framed, .. } => {
                        let handle     = env.atom_reg.acknowledge_or_else(image.clone(), || ui_assets.missing_texture.clone_weak());
                        let texture_id = ui_assets.texture_or_def(&handle);
                        let color      = env.atom_reg.acknowledge_or_else(color.clone(), || DynColor::CONST_FUCHSIA);
                        let rgba       = env.color_cache.simple_rgba(&color, loaded_palettes.as_ref(), palettes.as_ref(), shared_instances.as_ref());
                        let is_framed  = env.atom_reg.acknowledge_or_else(*is_framed, || false);
                        let elem       = egui::ImageButton::new(texture_id, size.unwrap_or(egui::Vec2::new(64., 64.)))
                            .tint(egui::Color32::from(rgba))
                            .frame(is_framed);
                        Some(match show_to {
                            ShowTo::TopLevel => {
                                egui::Area::new(format!("floating{:?}", texture_id)).show(env.ctx, |ui| {
                                    ui.add(elem)
                                }).inner
                            },
                            ShowTo::Ui(ui) => {
                                ui.add(elem)
                            },
                        })
                    },
                    ElemKind::Label { text } => {
                        match show_to {
                            ShowTo::TopLevel => { warn!("Label not supported at TopLevel"); None },
                            ShowTo::Ui(ui) => {
                                Some(ui.label(env.atom_reg.acknowledge_layout_job(
                                    &text,
                                    |c| env.color_cache
                                        .simple_rgba(&c, loaded_palettes.as_ref(), palettes.as_ref(), shared_instances.as_ref()),
                                    || "ERROR".to_string()
                                )))
                            },
                        }
                    },
                    ElemKind::Menu { children } => {
                        match show_to {
                            ShowTo::TopLevel => { warn!("Menu not supported at TopLevel"); None },
                            ShowTo::Ui(ui) => {
                                Some(egui::menu::bar(ui, |ui| {
                                    for (menu_name, children) in children.iter() {
                                        let menu_name = env.atom_reg.acknowledge_or_else(menu_name.clone(), || "ERROR".to_string());
                                        ui.menu_button(menu_name, |ui| {
                                            for child in children.iter() {
                                                (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                            }
                                        });
                                    }
                                }).response)
                            },
                        }
                    },
                    ElemKind::SidePanel { id, side, children } => {
                        let elem = egui::SidePanel::new(*side, *id);
                        Some(match show_to {
                            ShowTo::TopLevel => {
                                elem.show(env.ctx, |ui| {
                                    for child in children.iter() {
                                        (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                    }
                                })
                            },
                            ShowTo::Ui(ui) => {
                                elem.show_inside(ui, |ui| {
                                    for child in children.iter() {
                                        (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                    }
                                })
                            },
                        }.response)
                    },
                    ElemKind::VerticalPanel { id, side, children } => {
                        let elem = egui::TopBottomPanel::new(*side, *id);
                        Some(match show_to {
                            ShowTo::TopLevel => {
                                elem.show(env.ctx, |ui| {
                                    for child in children.iter() {
                                        (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                    }
                                })
                            },
                            ShowTo::Ui(ui) => {
                                elem.show_inside(ui, |ui| {
                                    for child in children.iter() {
                                        (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                    }
                                })
                            },
                        }.response)
                    },
                    ElemKind::Window { id, title, is_closeable, is_open, is_resizable, has_scrollx, has_scrolly, children } => {
                        let is_open_atom = env.atom_reg.acknowledge_or_else(*is_open, || true);

                        if is_open_atom {
                            let is_resizable  = env.atom_reg.acknowledge_or_else(*is_resizable, || false);
                            let is_closeable  = env.atom_reg.acknowledge_or_else(*is_closeable, || false);
                            let has_scrollx   = env.atom_reg.acknowledge_or_else(*has_scrollx, || false);
                            let has_scrolly   = env.atom_reg.acknowledge_or_else(*has_scrolly, || false);
                            let mut window = match title {
                                Some(t) => egui::Window::new(env.atom_reg.acknowledge_layout_job(
                                    &t,
                                    |c| env.color_cache
                                        .simple_rgba(&c, loaded_palettes.as_ref(), palettes.as_ref(), shared_instances.as_ref()),
                                    || "ERROR".to_string()
                                )),
                                None => egui::Window::new("")
                            }.id(*id)
                                .resizable(is_resizable)
                                .scroll2([has_scrollx, has_scrolly]);
    
                            let mut is_open_bool = is_open_atom;
                            if is_closeable {
                                if let OrAtom::Val(b) = is_open {
                                    is_open_bool = env.ui_cache.elem_is_open.get(id).cloned().unwrap_or(*b);
                                }
                                window = window.open(&mut is_open_bool);
                            }
                            if title.is_none() {
                                window = window.title_bar(false);
                            }
    
                            let opt_response = window.show(env.ctx, |ui| {
                                for child in children.iter() {
                                    (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                }
                            });
                            if is_open_atom != is_open_bool && let OrAtom::Atom(a) = is_open {
                                env.atom_reg.set(a.clone(), is_open_bool);
                            } else {
                                env.ui_cache.elem_is_open.insert(*id, is_open_bool);
                            }
                            opt_response.map(|r| r.response)
                        } else { None }
                    },
                };

                if let Some(response) = response {
                    if response.clicked() {
                        call_on_click(&elem.on_click, env.inst_ref);
                    }
                    if let Some(inst) = &elem.tooltip {
                        response.on_hover_text(env.atom_reg.acknowledge_layout_job(
                            &inst,
                            |c| env.color_cache
                                .simple_rgba(&c, loaded_palettes.as_ref(), palettes.as_ref(), shared_instances.as_ref()),
                            || "ERROR".to_string()
                        ));
                    }
                }
            }
        },
    };
    for (handle, container) in containers.iter() {
        if visibilities.0.contains(&handle) {
            let lua_inst = shared_instances.instances.get(&container.script_id).unwrap();
            let inst_ref = lua_inst.result.as_ref().unwrap();
            for elem in container.elems.iter() {
                let mut env = RunElemEnv {
                    inst_ref,
                    atom_reg: &mut atom_reg,
                    color_cache: &mut color_cache,
                    ctx: egui_ctx.ctx_mut(),
                    ui_cache: &mut ui_cache,
                };
                (run_elem.f)(&run_elem, elem, &mut env, ShowTo::TopLevel);
            }
        }
    }
}