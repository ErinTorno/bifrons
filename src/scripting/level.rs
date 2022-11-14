use ::std::sync::Mutex;

use bevy::{prelude::*, transform::TransformBundle};
use bevy_mod_scripting::{prelude::*, lua::api::bevy::{LuaWorld, LuaEntity, LuaVec3}};
use mlua::Lua;

use crate::{system::{level::{LoadedLevel}, common::{ToInitHandle, fix_missing_extension}}, data::level::{LevelPiece, LevelPieceLoader, Level, LevelLoader}};

use super::LuaHandle;

#[derive(Default)]
pub struct LevelAPIProvider;

impl APIProvider for LevelAPIProvider {
    type APITarget = Mutex<Lua>;
    type DocTarget = LuaDocFragment;
    type ScriptContext = Mutex<Lua>;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        attach_level_lua(ctx).map_err(ScriptError::new_other)?;
        Ok(())
    }
}

fn attach_level_lua(ctx: &mut Lua) -> Result<(), mlua::Error> {
    let table = ctx.create_table()?;
    table.set("change", ctx.create_function(|ctx, path: String| {
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let path = fix_missing_extension::<LevelLoader>(path);
            let handle: Handle<Level> = {
                let w = world.read();
                let asset_server = w.get_resource::<AssetServer>().unwrap();
                asset_server.load(&path)
            };
            let mut w = world.write();
            w.remove_resource::<LoadedLevel>();
            Ok(())
        })?
    )?;
    table.set("handle_of", ctx.create_function(|ctx, entity: LuaEntity| {
        let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
        let w = world.read();
        if let Some(handle) = w.get::<Handle<Level>>(entity.inner()?) {
            Ok(Some(LuaHandle::from(handle.clone())))
        } else { Ok(None) }
    })?)?;
    table.set("name", ctx.create_function(|ctx, ()| {
            match ctx.globals().get::<_, LuaWorld>("world").unwrap().read().get_resource::<LoadedLevel>() {
                Some(ll) => Ok(Some(ll.level.name.to_string())),
                None     => Ok(None)
            }
        })?
    )?;
    table.set("spawn_piece", ctx.create_function(|ctx, (asset, table): (String, LuaTable)| {
            let translation = if let Some(pos) = table.get::<_, Option<LuaVec3>>("pos")? {
                pos.inner()?
            } else { Vec3::ZERO };
            let rotation = if let Some(rot) = table.get::<_, Option<LuaVec3>>("rotation")? {
                rot.inner()?
            } else { Vec3::ZERO };
            let is_revealed = table.get::<_, Option<bool>>("reveal")?.unwrap_or(false);
            let name: String = table.get("name")?;

            let file = fix_missing_extension::<LevelPieceLoader>(asset);
            let world = ctx.globals().get::<_, LuaWorld>("world").unwrap();
            let handle: Handle<LevelPiece> = {
                let w = world.read();
                let asset_server = w.get_resource::<AssetServer>().unwrap();
                asset_server.load(&file)
            };
            let id = {
                let mut w = world.write();
                let mut entity = w.spawn();
                let id = entity.id();

                entity.insert(ToInitHandle(handle))
                    .insert(Name::from(name))
                    .insert_bundle(VisibilityBundle {
                        visibility: Visibility { is_visible: is_revealed },
                        ..VisibilityBundle::default()
                    })
                    .insert_bundle(TransformBundle {
                        local: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z)).with_translation(translation),
                        ..default()
                    });
                id
            };

            if let Some(parent) = table.get::<_, Option<LuaEntity>>("parent")? {
                let mut w = world.write();
                if let Some(mut parent_entity) = w.get_entity_mut(parent.inner()?) {
                    parent_entity.push_children(&[id]);
                }
            }
            Ok(LuaEntity::new(id))
        })?
    )?;
    ctx.globals().set("Level", table)?;
    Ok(())
}