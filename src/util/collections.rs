use std::{hash::Hash, collections::HashSet};

pub trait Singleton {
    type Item;

    fn singleton(item: Self::Item) -> Self;
}

impl<T> Singleton for std::collections::HashSet<T> where T: Eq + Hash {
    type Item = T;

    fn singleton(item: Self::Item) -> Self {
        let mut hs = HashSet::new();
        hs.insert(item);
        hs
    }
}

impl<T> Singleton for bevy::utils::HashSet<T> where T: Eq + Hash {
    type Item = T;

    fn singleton(item: Self::Item) -> Self {
        let mut hs = bevy::utils::HashSet::new();
        hs.insert(item);
        hs
    }
}