use std::{hash::Hash, collections::{HashSet, HashMap}};

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

impl<K, V> Singleton for std::collections::HashMap<K, V> where K: Eq + Hash {
    type Item = (K, V);
    fn singleton(item: Self::Item) -> Self {
        let mut hm = HashMap::new();
        hm.insert(item.0, item.1);
        hm
    }
} 