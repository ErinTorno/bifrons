use std::{cmp::*, collections::HashMap, hash::Hash};

use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use serde::*;


#[derive(Clone, Debug, Deserialize, PartialEq, Reflect, Serialize)]
pub enum JoystickSide {
    Left,
    Right,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Reflect, Serialize)]
pub enum ScrollWheel {
    Up,
    Down,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Reflect, Serialize)]
pub enum InputCode {
    Key      (KeyCode),
    Mouse    (MouseButton),
    Scroll   (ScrollWheel),
    Gamepad  (GamepadButtonType),
    Shoulder (GamepadAxisType),
    Joystick { side: JoystickSide, angle: f32, sensitivity: f32 },
}

#[derive(Clone, Debug, Inspectable, PartialEq, Reflect)]
pub enum InputData {
    Binary      { pressed: bool },
    Analog      { state: f32, },
    Directional { state: f32, angle: f32 },
    VecChain    { vecs: Vec<Vec2> },
}
impl InputData {
    pub fn is_empty(&self) -> bool {
        match self {
            InputData::Binary { pressed }        => !pressed,
            InputData::Analog { state }          => *state == 0.,
            InputData::Directional { state, .. } => *state == 0.,
            InputData::VecChain { vecs }         => vecs.is_empty(),
        }
    }

    pub fn power(&self) -> f32 {
        match self {
            InputData::Binary { pressed }       => if *pressed { 1. } else { 0. },
            InputData::Analog { state }          => *state,
            InputData::Directional { state, .. } => *state,
            InputData::VecChain { vecs }   => vecs.len() as f32,
        }
    }
}
impl Default for InputData {
    fn default() -> Self {
        InputData::Binary { pressed: false }
    }
}

#[derive(Clone, Copy, Debug, Default, Inspectable, PartialEq, Reflect)]
pub enum InputTime {
    #[default]
    NotPressed,
    JustPressed  { secs: f64 },
    Held         { secs: f64 },
    JustReleased { secs: f64 },
}
impl PartialOrd for InputTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            InputTime::NotPressed   => Some(if *other == InputTime::NotPressed { Ordering::Equal } else { Ordering::Less }),
            InputTime::Held { secs: s1 } => match other {
                InputTime::Held { secs: s2 } => s1.partial_cmp(s2),
                _ => Some(Ordering::Greater),
            },
            InputTime::JustReleased { secs: s1 } => match other {
                InputTime::Held {..} => Some(Ordering::Less),
                InputTime::JustPressed { secs: s2 } => s1.partial_cmp(s2),
                _ =>                  Some(Ordering::Greater),
            },
            InputTime::JustPressed  { secs: s1 } => match other {
                InputTime::Held {..} => Some(Ordering::Less),
                InputTime::JustReleased { secs: s2 } => s1.partial_cmp(s2),
                _ =>                  Some(Ordering::Greater),
            },
        }
    }
}
impl InputTime {
    // pub fn cmp_priority<F>(self, other: Self, cmp: F) -> Self where F: Fn(f64, f64) -> Ordering {
    //     match self {
    //         InputTime::NotPressed   => other,
    //         InputTime::Held { secs: s1 } => match other {
    //             InputTime::Held { secs: s2 } => if cmp(s1, s2) == Ordering::Greater { self } else { other },
    //             _ => self,
    //         },
    //         InputTime::JustReleased {..} => match other {
    //             InputTime::Held {..} => other,
    //             InputTime::JustPressed { secs } => InputTime::Held { secs },
    //             _ =>                  self,
    //         },
    //         InputTime::JustPressed  {..} => match other {
    //             InputTime::Held {..} => other,
    //             InputTime::JustReleased { secs } => InputTime::Held { secs },
    //             _ =>                  self,
    //         },
    //     }
    // }

    pub fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }

    pub fn from_input<T>(input: &Input<T>, code: T, secs: f64) -> InputTime where T: Copy + Eq + Hash {
        if input.just_pressed(code) {
            InputTime::JustPressed { secs } // todo get analog control state
        } else if input.just_released(code) {
            InputTime::JustReleased { secs  }
        } else if input.pressed(code) {
            InputTime::Held { secs }
        } else {
            InputTime::NotPressed
        }
    }
}

#[derive(Clone, Debug, Default, Inspectable, PartialEq, Reflect)]
pub struct InputTS {
    pub data: InputData,
    pub time: InputTime,
}
impl InputTS {
    pub fn combine(self, other: Self) -> Self {
        if self.data.is_empty() {
            other
        } else if other.data.is_empty() {
            self
        } else {
            match self.time.cmp(&other.time) {
                Ordering::Equal => {
                    let sp = self.data.power();
                    let op = other.data.power();
                    if sp >= op { self } else { other }
                },
                Ordering::Less => {
                    InputTS { data: other.data, time: self.time }
                },
                Ordering::Greater => {
                    InputTS { data: self.data, time: self.time }
                },
            }
        }
    }

    pub fn power(&self) -> f32 {
        match self.time {
            InputTime::NotPressed => 0.,
            _ => self.data.power(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Inspectable, PartialEq, Reflect)]
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
impl ActionState {
    pub fn input_ts<S>(&self, s: S) -> InputTS where S: AsRef<str> {
        self.inputs.get(s.as_ref()).cloned().unwrap_or(InputTS::default())
    }
}

#[derive(Clone, Component, Debug, Deserialize, PartialEq, Serialize)]
pub struct InputMap {
    pub actions: HashMap<String, Vec<InputCode>>,
    pub cam_speed: f32,
    pub zoom_speed: f32,
}
impl Default for InputMap {
    fn default() -> Self {
        let mut actions = HashMap::new();
        actions.insert("forward".into(),    vec![InputCode::Key(KeyCode::W)]);
        actions.insert("back".into(),       vec![InputCode::Key(KeyCode::S)]);
        actions.insert("left".into(),       vec![InputCode::Key(KeyCode::A)]);
        actions.insert("right".into(),      vec![InputCode::Key(KeyCode::D)]);
        actions.insert("rise".into(),       vec![InputCode::Key(KeyCode::Space)]);
        actions.insert("fall".into(),       vec![InputCode::Key(KeyCode::LControl)]);
        actions.insert("cam_switch".into(), vec![InputCode::Key(KeyCode::LShift), InputCode::Mouse(MouseButton::Middle)]);
        actions.insert("select".into(),     vec![InputCode::Key(KeyCode::E),  InputCode::Mouse(MouseButton::Left)]);
        actions.insert("drag".into(),       vec![InputCode::Mouse(MouseButton::Right)]);
        actions.insert("zoom_in".into(),    vec![InputCode::Key(KeyCode::Plus),  InputCode::Scroll(ScrollWheel::Up)]);
        actions.insert("zoom_out".into(),   vec![InputCode::Key(KeyCode::Minus), InputCode::Scroll(ScrollWheel::Down)]);
        InputMap {
            actions,
            cam_speed: 10.,
            zoom_speed: 20.,
        }
    }
}