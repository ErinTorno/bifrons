use bevy::{prelude::*};
use bevy_mod_scripting::prelude::{ScriptCollection, LuaFile, Script};
use crate::{data::{item::Item, material::TexMatInfo}, scripting::LuaScriptVars};

use super::{texture::{MaterialColors, Background}, common::ToInitHandle};

#[derive(Clone, Debug, Default)]
pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system(spawn_item)
        ;
    }
}

pub fn spawn_item(
    mut commands:     Commands,
    mut mat_colors:   ResMut<MaterialColors>,
    asset_server:     Res<AssetServer>,
    background:       Res<Background>,
    mut tex_mat_info: ResMut<TexMatInfo>,
    items:            Res<Assets<Item>>,
    mut meshes:       ResMut<Assets<Mesh>>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
    mut to_spawn:     Query<(Entity, &ToInitHandle<Item>, &mut LuaScriptVars)>,
) {
    for (entity, ToInitHandle(handle), mut script_vars) in to_spawn.iter_mut() {
        if let Some(item) = items.get(handle) {
            let entity = commands.entity(entity)
                .remove::<ToInitHandle<Item>>()
                .id();
            let loaded = item.animation.add_parts(&mut commands.entity(entity), mat_colors.as_mut(), &asset_server, &mut tex_mat_info, &background, meshes.as_mut(), materials.as_mut());
            commands.entity(entity).insert(loaded);

            if !item.scripts.is_empty() {
                script_vars.merge(&item.script_vars);
                commands.entity(entity)
                    .insert(ScriptCollection::<LuaFile> {
                        scripts: item.scripts.iter().map(|path| {
                            let handle = asset_server.load::<LuaFile, _>(path);
                            Script::<LuaFile>::new(path.clone(), handle)
                        }).collect()
                    });
            }
        }
    }
}