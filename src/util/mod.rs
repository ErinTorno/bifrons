use bevy::{prelude::{Color, Component}, ecs::system::EntityCommands};
use bevy_inspector_egui::prelude::*;

pub mod collections;
pub mod serialize;
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

pub trait InsertableWithPredicate {
    fn insert_if<F, C>(&mut self, b: bool, make_component: F) -> &mut Self where F: FnOnce() -> C, C: Component;
}

impl<'w, 's, 'a> InsertableWithPredicate for EntityCommands<'w, 's, 'a> {
    fn insert_if<F, C>(&mut self, b: bool, make_component: F) -> &mut Self where F: FnOnce() -> C, C: Component {
        if b {
            self.insert(make_component());
        }
        self
    }
}

pub trait RoughlyEq<T> {
    type Epsilon;
    fn roughly_eq(self, that: T, epsilon: Self::Epsilon) -> bool;
}
impl RoughlyEq<f32> for f32 {
    type Epsilon = f32;

    fn roughly_eq(self, that: f32, epsilon: Self::Epsilon) -> bool {
        self >= that - epsilon && self <= that + epsilon
    }
}
impl RoughlyEq<f64> for f32 {
    type Epsilon = f64;

    fn roughly_eq(self, that: f64, epsilon: Self::Epsilon) -> bool {
        self as f64 >= that - epsilon && self as f64 <= that + epsilon
    }
}
impl RoughlyEq<f64> for f64 {
    type Epsilon = f64;

    fn roughly_eq(self, that: f64, epsilon: Self::Epsilon) -> bool {
        self >= that - epsilon && self <= that + epsilon
    }
}
impl RoughlyEq<f32> for f64 {
    type Epsilon = f64;

    fn roughly_eq(self, that: f32, epsilon: Self::Epsilon) -> bool {
        self >= that as f64 - epsilon && self <= that as f64 + epsilon
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