use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

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

#[derive(Clone, Component, Copy, Debug)]
pub struct ToInitWith<T: ?Sized, P> {
    to_init: ToInit<T>,
    params: P,
}

impl<T, P> Default for ToInitWith<T, P> where P: Default {
    fn default() -> Self {
        ToInitWith { to_init: ToInit::default(), params: P::default() }
    }
}

pub fn easy_hash<H>(h: &H) -> u64 where H: Hash {
    let mut hasher = DefaultHasher::new();
    h.hash(&mut hasher);
    hasher.finish()
}

pub fn fix_missing_extension<T>(file: String) -> String where T: Default + AssetLoader {
    let path = Path::new(&file);
    if path.extension().is_none() {
        if let Some(ext) = T::default().extensions().iter().next() {
            format!("{}.{}", file, ext)
        } else { file }
    } else { file }
}