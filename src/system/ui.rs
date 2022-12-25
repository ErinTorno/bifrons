use std::collections::{HashMap};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use iyes_loopless::prelude::{IntoConditionalSystem};
use mlua::prelude::*;

use crate::{scripting::{ui::{elem::*, atom::{LuaAtomRegistry}}}, data::{lua::InstanceRef, font::Font, palette::{Palette, ColorCache, LoadedPalettes, DynColor}}, system::lua::LuaInstance};

use super::lua::SharedInstances;

#[derive(Clone, Debug, Default)]
pub struct ScriptingUiPlugin;

impl Plugin for ScriptingUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .init_resource::<LoadingUIAssets>()
            .init_resource::<UIAssets>()
            .init_resource::<VisibleContainers>()
            .add_startup_system(setup_default_ui_assets)
            .add_system(init_ui_assets.run_if_resource_exists::<LoadingUIAssets>())
            .add_system(run_containers)
        ;
    }
}

#[derive(Clone, Copy, Default, Resource)]
pub struct LoadingUIAssets;

#[derive(Clone, Default, Resource)]
pub struct UIAssets {
    pub font_mono:          Handle<Font>,
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

pub fn is_ui_focused(mut egui_ctx: ResMut<EguiContext>) -> bool {
    let ctx = egui_ctx.ctx_mut();
    ctx.is_using_pointer() || ctx.is_pointer_over_area()
}

pub fn setup_default_ui_assets(
    asset_server:  Res<AssetServer>,
    mut ui_assets: ResMut<UIAssets>,
    mut egui_ctx:  ResMut<EguiContext>,
) {
    ui_assets.font_mono = asset_server.load("fonts/UbuntuMono-R.ttf");
    ui_assets.missing_texture = asset_server.load("missing.ktx2");
    let missing_handle = ui_assets.missing_texture.clone_weak();
    ui_assets.missing_texture_id = ui_assets.texture_id_or_insert(missing_handle, egui_ctx.as_mut());
}

pub fn init_ui_assets(
    mut commands:  Commands,
    fonts:         Res<Assets<Font>>,
    ui_assets:     Res<UIAssets>,
    mut egui_ctx:  ResMut<EguiContext>,
) {
    if let Some(mono_font) = fonts.get(&ui_assets.font_mono) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert("mono".to_string(), mono_font.0.clone());

        fonts.families.entry(egui::FontFamily::Monospace   ).or_default().insert(0, "mono".to_string());
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "mono".to_string());

        let ctx = egui_ctx.ctx_mut();
        ctx.set_fonts(fonts);

        commands.remove_resource::<LoadingUIAssets>()
    }
}

enum ShowTo<'a> { TopLevel, Ui(&'a mut egui::Ui) }

pub fn run_containers(
    containers:       Res<Assets<Container>>,
    palettes:         Res<Assets<Palette>>,
    mut color_cache:  ResMut<ColorCache>,
    loaded_palettes:  Res<LoadedPalettes>,
    shared_instances: Res<SharedInstances>,
    mut egui_ctx:     ResMut<EguiContext>,
    mut atom_reg:     ResMut<LuaAtomRegistry>,
    ui_assets:        Res<UIAssets>,
    visibilities:     Res<VisibleContainers>,
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
        color_cache: &'a mut ColorCache
    }
    struct RunElem<'a> { f: &'a dyn Fn(&RunElem, &Elem, &mut RunElemEnv, ShowTo) -> () }
    let run_elem = RunElem {
        f: &|run_elem, elem, env, show_to| {
            let size = env.atom_reg.acknowledge_option(elem.size.clone());
            let size = size.map(|s| egui::Vec2::new(s.x, s.y));
            match &elem.kind {
                ElemKind::Horizontal { children } => {
                    match show_to {
                        ShowTo::TopLevel => { warn!("Horizontal not supported at TopLevel"); },
                        ShowTo::Ui(ui) => {
                            ui.horizontal(|ui| {
                                for child in children.iter() {
                                    (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                }
                            });
                        },
                    }
                },
                ElemKind::Vertical { children } => {
                    match show_to {
                        ShowTo::TopLevel => { warn!("Vertical not supported at TopLevel"); },
                        ShowTo::Ui(ui) => {
                            ui.vertical(|ui| {
                                for child in children.iter() {
                                    (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                }
                            });
                        },
                    }
                },
                ElemKind::ImageButton { on_click, image, color, is_framed, .. } => {
                    let handle     = env.atom_reg.acknowledge_or_else(image.clone(), || ui_assets.missing_texture.clone_weak());
                    let texture_id = ui_assets.texture_or_def(&handle);
                    let color      = env.atom_reg.acknowledge_or_else(color.clone(), || DynColor::CONST_FUCHSIA);
                    let rgba       = env.color_cache.simple_rgba(&color, loaded_palettes.as_ref(), palettes.as_ref(), shared_instances.as_ref());
                    let is_framed  = env.atom_reg.acknowledge_or_else(*is_framed, || false);
                    let elem       = egui::ImageButton::new(texture_id, size.unwrap_or(egui::Vec2::new(64., 64.)))
                        .tint(egui::Color32::from(rgba))
                        .frame(is_framed);
                    match show_to {
                        ShowTo::TopLevel => {
                            egui::Area::new(format!("floating{:?}", texture_id)).show(env.ctx, |ui| {
                                if ui.add(elem).clicked() {
                                    call_on_click(on_click, env.inst_ref);
                                }
                            });
                        },
                        ShowTo::Ui(ui) => {
                            if ui.add(elem).clicked() {
                                call_on_click(on_click, env.inst_ref);
                            }
                        },
                    }
                },
                ElemKind::SidePanel { name, side, children } => {
                    let elem = egui::SidePanel::new(*side, name.clone());
                    match show_to {
                        ShowTo::TopLevel => {
                            elem.show(env.ctx, |ui| {
                                for child in children.iter() {
                                    (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                }
                            });
                        },
                        ShowTo::Ui(ui) => {
                            elem.show_inside(ui, |ui| {
                                for child in children.iter() {
                                    (run_elem.f)(run_elem, child, env, ShowTo::Ui(ui));
                                }
                            });
                        },
                    }
                },
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
                };
                (run_elem.f)(&run_elem, elem, &mut env, ShowTo::TopLevel);
            }
        }
    }
}