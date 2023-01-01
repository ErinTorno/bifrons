use bevy::{prelude::{Component, Handle}, asset::{AssetLoader, Asset}};
use ghost::phantom;
use std::path::Path;

/// A component that marks an Entity as needing initialization logic from some system before use
#[derive(Clone, Component, Copy, Debug)]
#[phantom]
pub struct ToInit<T: ?Sized>;
impl<T> Default for ToInit<T>  {
    fn default() -> Self {
        ToInit
    }
}

#[derive(Clone, Component, Default, Debug)]
pub struct ToInitHandle<T>(pub Handle<T>) where T: Asset;
impl<T> ToInitHandle<T> where T: Asset {
    pub fn new(handle: Handle<T>) -> Self {
        ToInitHandle(handle)
    }
}

pub fn fix_missing_extension<T>(file: String) -> String where T: Default + AssetLoader {
    let path = Path::new(&file);
    if path.extension().is_none() {
        if let Some(ext) = T::default().extensions().iter().next() {
            format!("{}.{}", file, ext)
        } else { file }
    } else { file }
}