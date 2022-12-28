
use std::collections::HashMap;

use bevy::asset::FileAssetIo;
use mlua::prelude::*;
use rfd::FileDialog;

use super::LuaMod;

pub struct FileAPI;
impl LuaMod for FileAPI {
    fn mod_name() -> &'static str { "File" }

    fn register_defs(lua: &Lua, table: &mut LuaTable) -> Result<(), mlua::Error> {
        table.set("dialog", lua.create_function(|_, table: LuaTable| {
            let mut dialog = FileDialog::new();

            if let Some(filters) = table.get::<_, Option<HashMap<String, Vec<String>>>>("filters")? {
                for (name, exts) in filters.iter() {
                    let extensions: Vec<_> = exts.iter().map(String::as_str).collect();
                    dialog = dialog.add_filter(&name, &extensions);
                }
            }

            let mut buf = FileAssetIo::get_base_path();
            if let Some(directory) = table.get::<_, Option<String>>("directory")? {
                buf.push(directory);
            }
            dialog = dialog.set_directory(buf);
            Ok(dialog.pick_file().map(|p| p.to_string_lossy().to_string()))
        })?)?;
        Ok(())
    }
}