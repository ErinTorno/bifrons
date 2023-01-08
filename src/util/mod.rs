use std::{cmp::Ordering, hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};

use bevy::{prelude::{Color}};
use bevy_inspector_egui::prelude::*;
use ron::{Options, extensions::Extensions};

pub fn ron_options() -> Options {
    Options::default().with_default_extension(Extensions::all())
}

pub mod collections;

pub fn easy_hash<H>(h: &H) -> u64 where H: Hash {
    let mut hasher = DefaultHasher::new();
    h.hash(&mut hasher);
    hasher.finish()
}

pub trait IntoHex {
    fn into_hex(&self) -> String;
}
impl IntoHex for Color {
    fn into_hex(&self) -> String {
        let c = self.as_rgba_f32();
        if c[3] == 1. {
            format!("#{:02x}{:02x}{:02x}", (c[0] * 255.) as u8, (c[1] * 255.) as u8, (c[2] * 255.) as u8)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", (c[0] * 255.) as u8, (c[1] * 255.) as u8, (c[2] * 255.) as u8, (c[3] * 255.) as u8)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Inspectable, PartialEq)]
pub struct Timestamped<T> {
    pub time:  f64,
    pub value: T,
}

pub fn pair_clone<A, B>((a, b): (&A, &B)) -> (A, B) where A: Clone, B: Clone {
    (a.clone(), b.clone())
}

#[derive(Clone, Copy, Debug)]
pub struct Roughly<T>(pub T);

impl<T> Eq for Roughly<T> where T: Clone + RoughlyEq<T> {
}
impl<T> PartialEq for Roughly<T> where T: Clone + RoughlyEq<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.roughly_eq(&other.0)
    }
}
impl<T> Ord for Roughly<T> where T: Clone + PartialOrd + RoughlyEq<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let this = &self.0;
        let that = &other.0;
        if this.roughly_eq(that) {
            Ordering::Equal
        } else if this < that {
            Ordering::Less
        } else if this > that {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
impl<T> PartialOrd for Roughly<T> where T: Clone + PartialOrd + RoughlyEq<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let this = &self.0;
        let that = &other.0;
        if this.roughly_eq(that) {
            Some(Ordering::Equal)
        } else {
            this.partial_cmp(&that)
        }
    }
}

pub trait RoughlyEq<T> {
    type Epsilon;

    fn default_epsilon() -> Self::Epsilon;

    fn roughly_eq_within(&self, that: &T, epsilon: Self::Epsilon) -> bool;

    fn roughly_eq(&self, that: &T) -> bool {
        self.roughly_eq_within(that, Self::default_epsilon())
    }
}
impl RoughlyEq<f32> for f32 {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon { 0.000001 }

    fn roughly_eq_within(&self, that: &f32, epsilon: Self::Epsilon) -> bool {
        *self >= *that - epsilon && *self <= *that + epsilon
    }
}
impl RoughlyEq<f64> for f32 {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon { 0.000001 }

    fn roughly_eq_within(&self, that: &f64, epsilon: Self::Epsilon) -> bool {
        *self as f64 >= *that - epsilon && *self as f64 <= *that + epsilon
    }
}
impl RoughlyEq<f64> for f64 {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon { 0.000001 }

    fn roughly_eq_within(&self, that: &f64, epsilon: Self::Epsilon) -> bool {
        *self >= *that - epsilon && *self <= *that + epsilon
    }
}
impl RoughlyEq<f32> for f64 {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon { 0.000001 }

    fn roughly_eq_within(&self, that: &f32, epsilon: Self::Epsilon) -> bool {
        *self >= *that as f64 - epsilon && *self <= *that as f64 + epsilon
    }
}
pub trait RoughToBits<T> {
    fn rough_to_bits(self) -> T;
}
impl RoughToBits<u32> for f32 {
    fn rough_to_bits(self) -> u32 {
        if self.is_nan() {
            f32::NAN
        } else {
            (self * 100000.).round() / 100000.
        }.to_bits()
    }
}
impl RoughToBits<u64> for f64 {
    fn rough_to_bits(self) -> u64 {
        if self.is_nan() {
            f64::NAN
        } else {
            (self * 100000.).round() / 100000.
        }.to_bits()
    }
}