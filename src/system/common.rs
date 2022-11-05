use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use bevy::prelude::Component;
use bevy_inspector_egui::Inspectable;
use ghost::phantom;

/// A component that marks an Entity as needing initialization logic from some system before use
#[derive(Clone, Component, Copy, Debug, Inspectable)]
#[phantom]
pub struct ToInit<T: ?Sized>;

impl<T> Default for ToInit<T>  {
    fn default() -> Self {
        ToInit
    }
}

#[derive(Clone, Component, Copy, Debug, Inspectable)]
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