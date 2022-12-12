use bevy::{prelude::*, transform::TransformBundle};
use mlua::prelude::*;

use crate::{system::{level::{LoadedLevel}, common::{ToInitHandle, fix_missing_extension}}, data::{level::{LevelPiece, LevelPieceLoader, Level, LevelLoader}, lua::LuaWorld}};

use super::{LuaMod, bevy_api::{LuaEntity, handle::LuaHandle, math::LuaVec3}};

#[derive(Default)]
pub struct LevelAPI;
impl LuaMod for LevelAPI {
    fn mod_name() -> &'static str { "Level" }
    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("change", lua.create_function(|lua, path: String| {
                let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                let path = fix_missing_extension::<LevelLoader>(path);
                let _handle: Handle<Level> = {
                    let w = world.read();
                    let asset_server = w.get_resource::<AssetServer>().unwrap();
                    asset_server.load(&path)
                };
                let mut w = world.write();
                w.remove_resource::<LoadedLevel>();
                // todo insert Loading resource
                Ok(())
            })?
        )?;
        table.set("handle_of", lua.create_function(|lua, entity: LuaEntity| {
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            if let Some(handle) = w.get::<Handle<Level>>(entity.0) {
                Ok(Some(LuaHandle::from(handle.clone())))
            } else { Ok(None) }
        })?)?;
        table.set("name", lua.create_function(|lua, ()| {
                match lua.globals().get::<_, LuaWorld>("world").unwrap().read().get_resource::<LoadedLevel>() {
                    Some(ll) => Ok(Some(ll.level.name.to_string())),
                    None     => Ok(None)
                }
            })?
        )?;
        table.set("spawn_piece", lua.create_function(|lua, (asset, table): (String, LuaTable)| {
                let translation = if let Some(pos) = table.get::<_, Option<LuaVec3>>("pos")? {
                    pos.0
                } else { Vec3::ZERO };
                let rotation = if let Some(rot) = table.get::<_, Option<LuaVec3>>("rotation")? {
                    rot.0
                } else { Vec3::ZERO };
                let is_revealed = table.get::<_, Option<bool>>("reveal")?.unwrap_or(false);
                let name: String = table.get("name")?;
    
                let file = fix_missing_extension::<LevelPieceLoader>(asset);
                let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
                let handle: Handle<LevelPiece> = {
                    let w = world.read();
                    let asset_server = w.get_resource::<AssetServer>().unwrap();
                    asset_server.load(&file)
                };
                let id = {
                    let mut w = world.write();
                    w.spawn((
                        Name::from(name),
                        ToInitHandle(handle),
                        TransformBundle {
                            local: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z)).with_translation(translation),
                            ..default()
                        },
                        VisibilityBundle {
                            visibility: Visibility { is_visible: is_revealed },
                            ..VisibilityBundle::default()
                        },
                    )).id()
                };
    
                if let Some(parent) = table.get::<_, Option<LuaEntity>>("parent")? {
                    let mut w = world.write();
                    if let Some(mut parent_entity) = w.get_entity_mut(parent.0) {
                        parent_entity.push_children(&[id]);
                    }
                }
                Ok(LuaEntity::new(id))
            })?
        )?;
        Ok(())
    }
}