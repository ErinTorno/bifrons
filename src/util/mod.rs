use bevy::prelude::Color;

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