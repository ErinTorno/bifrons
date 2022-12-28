use bevy::{asset::*, reflect::TypeUuid};
use bevy_egui::egui;

use crate::{scripting::{LuaMod, bevy_api::handle::LuaHandle}, data::lua::LuaWorld, system::common::fix_missing_extension};

#[derive(TypeUuid)]
#[uuid = "f0b75841-3643-4e9b-9d85-4e828417d7b0"]
pub struct UIFont {
    pub name: String,
    pub data: egui::FontData,
}
impl LuaMod for UIFont {
    fn mod_name() -> &'static str { "Font" }

    fn register_defs(lua: &mlua::Lua, table: &mut mlua::Table) -> Result<(), mlua::Error> {
        table.set("load", lua.create_function(|lua, path: String| {
            let path = fix_missing_extension::<FontLoader>(path);
            let world = lua.globals().get::<_, LuaWorld>("world").unwrap();
            let w = world.read();
            let asset_server = w.get_resource::<AssetServer>().unwrap();
            let handle: Handle<UIFont> = asset_server.load(&path);
            Ok(LuaHandle::from(handle))
        })?)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let font = UIFont {
                name: load_context.path().to_string_lossy().into(),
                data: egui::FontData::from_owned(bytes.to_vec()),
            };
            load_context.set_default_asset(LoadedAsset::new(font));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["otf", "ttf"]
    }
}