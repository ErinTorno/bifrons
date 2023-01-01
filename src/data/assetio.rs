use std::{collections::HashMap, path::{PathBuf, Path}, io::{Read, self}, fs, sync::Arc};

use bevy::{asset::{FileAssetIo, AssetIo, AssetIoError}, prelude::{Resource}};
use parking_lot::{RwLock, RwLockWriteGuard};
use std::fs::File;

pub type OverridesLock = Arc<RwLock<HashMap<PathBuf, PathBuf>>>;

#[derive(Resource)]
pub struct VirtualFileOverrides {
    pub overrides: OverridesLock,
    pub lua_path:  String,
}
impl VirtualFileOverrides {
    pub fn populate_files(&mut self, load_order: &Vec<Vec<&String>>) {
        fn visit_dirs(dir: &Path, root: &Path, overrides: &mut RwLockWriteGuard<HashMap<PathBuf, PathBuf>>) -> io::Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dirs(&path, root, overrides)?;
                    } else if let Some(filename) = path.file_name()
                           && !filename.to_string_lossy().ends_with(".mod.ron") {
                        let stripped = path.as_path().strip_prefix(root)
                            .map_err(|e| io::Error::other(e))?;
                        overrides.insert(PathBuf::from(stripped), path);
                    }
                }
            }
            Ok(())
        }
        let mut lua_paths = Vec::<String>::new();
        let mut overrides = self.overrides.write();
        overrides.clear();
        for wave in load_order.iter() {
            for mod_path in wave.iter() {
                if mod_path.as_str() == "assets" {
                    lua_paths.push(format!("{}/assets/?.lua", FileAssetIo::get_base_path().to_string_lossy()));
                } else {
                    let path_str = format!("{}/{}", FileAssetIo::get_base_path().to_string_lossy(), mod_path);
                    let path     = Path::new(&path_str);
                    visit_dirs(&path, &path, &mut overrides).unwrap();
                    lua_paths.push(format!("{}/?.lua", path_str));
                }
            }
        }
        lua_paths.reverse();
        self.lua_path = lua_paths.join(";");
    }
}

pub struct VirtualAssetIo {
    pub file_io: FileAssetIo,
    overrides:   OverridesLock,
}
impl VirtualAssetIo {
    pub fn new() -> VirtualAssetIo {
        VirtualAssetIo {
            file_io:   FileAssetIo::new("assets", false),
            overrides: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn overrides(&self) -> OverridesLock {
        self.overrides.clone()
    }

    pub fn root_path(&self) -> &PathBuf {
        self.file_io.root_path()
    }
}
impl AssetIo for VirtualAssetIo {
    fn get_metadata(&self, path: &std::path::Path) -> Result<bevy::asset::Metadata, bevy::asset::AssetIoError> {
        let read = self.overrides.read();
        let path: &Path = read.get(path).map(|p| &p as &Path).unwrap_or(&path);
        self.file_io.get_metadata(&path)
    }

    fn is_dir(&self, path: &std::path::Path) -> bool {
        let read = self.overrides.read();
        let path: &Path = read.get(path).map(|p| &p as &Path).unwrap_or(&path);
        self.file_io.is_dir(&path)
    }

    fn is_file(&self, path: &std::path::Path) -> bool {
        let read = self.overrides.read();
        let path: &Path = read.get(path).map(|p| &p as &Path).unwrap_or(&path);
        self.file_io.is_file(&path)
    }

    fn load_path<'a>(&'a self, path: &'a std::path::Path) -> bevy::utils::BoxedFuture<'a, Result<Vec<u8>, bevy::asset::AssetIoError>> {
        let read = self.overrides.read();
        let full_path: PathBuf = read.get(path).cloned().unwrap_or_else(|| self.root_path().join(path));
        Box::pin(async move {
            let mut bytes = Vec::new();
            match File::open(&full_path) {
                Ok(mut file) => {
                    file.read_to_end(&mut bytes)?;
                }
                Err(e) => {
                    return if e.kind() == std::io::ErrorKind::NotFound {
                        Err(AssetIoError::NotFound(full_path))
                    } else {
                        Err(e.into())
                    }
                }
            }
            Ok(bytes)
        })
    }

    fn read_directory(
            &self,
            path: &std::path::Path,
        ) -> Result<Box<dyn Iterator<Item = PathBuf>>, bevy::asset::AssetIoError> {
        let read = self.overrides.read();
        let path: &Path = read.get(path).map(|p| &p as &Path).unwrap_or(&path);
        self.file_io.read_directory(&path)
    }

    fn watch_for_changes(&self) -> Result<(), bevy::asset::AssetIoError> {
        self.file_io.watch_for_changes()
    }

    fn watch_path_for_changes(&self, path: &std::path::Path) -> Result<(), bevy::asset::AssetIoError> {
        let read = self.overrides.read();
        let path: &Path = read.get(path).map(|p| &p as &Path).unwrap_or(&path);
        self.file_io.watch_path_for_changes(&path)
    }
}