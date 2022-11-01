use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ColorOp {
    Add(f32),
    Div(f32),
    Mult(f32),
    Set(f32),
    Sub(f32),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ColorField {
    Alpha,
    Blue,
    Green,
    Hue,
    Lightness,
    Red,
    Saturation,
}