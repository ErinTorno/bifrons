use std::{cmp::*, collections::HashMap, hash::Hash, default};

use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use serde::*;


#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum JoystickSide {
    Left,
    Right,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ScrollWheel {
    Up   { sensitivity: f32 },
    Down { sensitivity: f32 },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum InputCode {
    Key      (KeyCode),
    Mouse    (MouseButton),
    Scroll   (ScrollWheel),
    Gamepad  (GamepadButtonType),
    Joystick { side: JoystickSide, angle: f32, sensitivity: f32 },
}

#[derive(Clone, Copy, Debug, Default, Inspectable, PartialEq)]
pub enum InputTS {
    #[default]
    NotPressed,
    JustPressed  { secs: f64 },
    Held         { secs: f64 },
    JustReleased { secs: f64 },
}
impl InputTS {
    pub fn max_priority(self, other: Self) -> Self {
        match self {
            InputTS::NotPressed   => other,
            InputTS::Held { secs: s1 } => match other {
                InputTS::Held { secs: s2 } => if s1 >= s2 { self } else { other }
                _ => self,
            },
            InputTS::JustReleased {..} => match other {
                InputTS::Held {..} => other,
                InputTS::JustPressed { secs} => InputTS::Held { secs },
                _ =>                  self,
            },
            InputTS::JustPressed  {..} => match other {
                InputTS::Held {..} => other,
                InputTS::JustReleased { secs} => InputTS::Held { secs },
                _ =>                  self,
            },
        }
    }

    pub fn from_input<T>(input: &Input<T>, code: T, secs: f64) -> InputTS where T: Copy + Eq + Hash {
        if input.just_pressed(code) {
            InputTS::JustPressed { secs }
        } else if input.just_released(code) {
            InputTS::JustReleased { secs }
        } else if input.pressed(code) {
            InputTS::Held { secs }
        } else {
            InputTS::NotPressed
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Inspectable, PartialEq)]
pub enum CameraType {
    #[default]
    Following, // follows the player
    Free,      // can freely move around
    Locked,    // no free roam or follow-the-player; cutscenes etc.
}
impl CameraType {
    pub fn toggle(self) -> Self {
        match self {
            CameraType::Free      => CameraType::Following,
            CameraType::Following => CameraType::Free,
            _ => self,
        }
    }
}

#[derive(Clone, Component, Debug, Default, Inspectable, PartialEq)]
pub struct ActionState {
    pub gamepad_id: Option<usize>,
    pub camera:     CameraType,
    pub inputs:     HashMap<String, InputTS>, // action -> last input start time
}

#[derive(Clone, Component, Debug, Deserialize, PartialEq, Serialize)]
pub struct InputMap {
    pub actions: HashMap<String, Vec<InputCode>>,
}
impl Default for InputMap {
    fn default() -> Self {
        let mut actions = HashMap::new();
        actions.insert("forward".into(),    vec![InputCode::Key(KeyCode::W)]);
        actions.insert("back".into(),       vec![InputCode::Key(KeyCode::S)]);
        actions.insert("left".into(),       vec![InputCode::Key(KeyCode::A)]);
        actions.insert("right".into(),      vec![InputCode::Key(KeyCode::D)]);
        actions.insert("cam_switch".into(), vec![InputCode::Key(KeyCode::LShift), InputCode::Mouse(MouseButton::Middle)]);
        actions.insert("select".into(),     vec![InputCode::Key(KeyCode::Space),  InputCode::Mouse(MouseButton::Left)]);
        actions.insert("drag".into(),       vec![InputCode::Mouse(MouseButton::Right)]);
        actions.insert("zoom_in".into(),    vec![InputCode::Key(KeyCode::Plus),  InputCode::Scroll(ScrollWheel::Up { sensitivity: 1. })]);
        actions.insert("zoom_out".into(),   vec![InputCode::Key(KeyCode::Minus), InputCode::Scroll(ScrollWheel::Down { sensitivity: 1. })]);
        InputMap {
            actions,
        }
    }
}