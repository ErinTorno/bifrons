use bevy::{prelude::{Color, Component}, reflect::Reflect, ecs::system::EntityCommands};
use bevy_inspector_egui::Inspectable;

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