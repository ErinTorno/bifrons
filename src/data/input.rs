use std::{cmp::*, collections::HashMap, hash::Hash};

use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use mlua::prelude::*;
use serde::*;

use crate::{scripting::{LuaMod, bevy_api::math::LuaVec2}, util::Timestamped};


#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum JoystickSide {
    Left,
    Right,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ScrollWheel {
    Up,
    Down,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum InputCode {
    Key         (KeyCode),
    MouseButton (MouseButton),
    MouseMotion { hold: MouseButton, sensitivity: f32 },
    Scroll      (ScrollWheel),
    Gamepad     (GamepadButtonType),
    Shoulder    (GamepadAxisType),
    Joystick    { side: JoystickSide, angle: f32, sensitivity: f32 },
}

#[derive(Clone, Debug, InspectorOptions, PartialEq)]
pub enum InputData {
    Binary      { pressed: bool },
    Analog      { state: f32, },
    Directional { state: f32, angle: f32 },
    VecChain    { vecs: Vec<Timestamped<Vec2>> },
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
            InputData::Binary { pressed }        => if *pressed { 1. } else { 0. },
            InputData::Analog { state }          => *state,
            InputData::Directional { state, .. } => *state,
            InputData::VecChain { vecs }         => if vecs.is_empty() { 0. } else { 2. },
        }
    }

    pub fn vec_chain(&self) -> Option<&Vec<Timestamped<Vec2>>> {
        match self {
            InputData::VecChain { vecs } => Some(vecs),
            _ => None,
        }
    }
}
impl Default for InputData {
    fn default() -> Self {
        InputData::Binary { pressed: false }
    }
}

#[derive(Clone, Copy, Debug, Default, InspectorOptions, PartialEq)]
pub enum InputTime {
    #[default]
    NotPressed,
    JustPressed  { secs: f64 },
    Held         { secs: f64, secs_start: f64 },
    JustReleased { secs: f64 },
}
impl InputTime {
    pub fn cmp_secs(&self, other: &Self) -> Ordering {
        match self {
            InputTime::NotPressed   => Some(if *other == InputTime::NotPressed { Ordering::Equal } else { Ordering::Less }),
            InputTime::Held { secs: s1,.. } => match other {
                InputTime::Held { secs: s2,.. } => s1.partial_cmp(s2),
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
        }.unwrap_or(Ordering::Equal)
    }

    pub fn combine(self, other: Self) -> Self {
        match self {
            InputTime::NotPressed   => if other == InputTime::NotPressed { self } else { other },
            InputTime::Held { secs: s1, secs_start: st1 } => match other {
                InputTime::Held { secs: s2, secs_start: st2 } => InputTime::Held { secs: s1.max(s2), secs_start: st1.min(st2) },
                _ => self,
            },
            InputTime::JustReleased { secs: s1 } => match other {
                InputTime::Held {..} => other,
                InputTime::JustPressed { secs: s2 } => if s1 >= s2 { self } else { other },
                _ => self,
            },
            InputTime::JustPressed  { secs: s1 } => match other {
                InputTime::Held {..} => other,
                InputTime::JustReleased { secs: s2 } => if s1 >= s2 { self } else { other },
                _ => self,
            },
        }
    }

    pub fn from_input<T>(input: &Input<T>, code: T, secs: f64) -> InputTime where T: Copy + Eq + Hash + Send + Sync {
        if input.just_pressed(code) {
            InputTime::JustPressed { secs } // todo get analog control state
        } else if input.just_released(code) {
            InputTime::JustReleased { secs  }
        } else if input.pressed(code) {
            InputTime::Held { secs, secs_start: secs }
        } else {
            InputTime::NotPressed
        }
    }
}

#[derive(Clone, Debug, Default, InspectorOptions, PartialEq)]
pub struct InputTS {
    pub data: InputData,
    pub time: InputTime,
}
impl InputTS {
    pub fn combine(self, other: Self) -> Self {
        if self.time == InputTime::NotPressed || self.data.is_empty() {
            other
        } else if other.time == InputTime::NotPressed || other.data.is_empty() {
            self
        } else {
            match self.time.cmp_secs(&other.time) {
                Ordering::Equal => {
                    let sp = self.data.power().abs();
                    let op = other.data.power().abs();
                    InputTS {
                        data: if sp >= op { self.data } else { other.data },
                        time: self.time.combine(other.time),
                    }
                },
                Ordering::Less => {
                    InputTS { data: other.data, time: self.time.combine(other.time) }
                },
                Ordering::Greater => {
                    InputTS { data: self.data, time: self.time.combine(other.time) }
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

    pub fn time_scaled_power(&self) -> f32 {
        match self.time {
            InputTime::NotPressed        => 0.,
            InputTime::JustPressed {..}  => self.data.power() / 3.,
            InputTime::Held { secs, secs_start }  => {
                let p = self.data.power();
                let m = (0.333 + (secs - secs_start) as f32 * 3.).min(1.).max(0.);
                m * p
            },
            InputTime::JustReleased {..} => self.data.power() / 3.,
        }
    }

    pub fn is_blank(&self) -> bool {
        match self.time {
            InputTime::NotPressed => true,
            _ => self.data.power() <= 0.,
        }
    }

    pub fn is_vec_chain(&self) -> bool {
        match self.data {
            InputData::VecChain {..} => true,
            _ => false,
        }
    }
}
impl LuaUserData for InputTS {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("angle", |_, this| match &this.data {
            InputData::Directional { angle, .. } => Ok(Some(*angle)),
            _ => Ok(None),
        });
        fields.add_field_method_get("inputstage", |_, this| Ok(match this.time {
            InputTime::NotPressed        => "not_pressed".to_string(),
            InputTime::Held {..}         => "held".to_string(),
            InputTime::JustPressed {..}  => "just_pressed".to_string(),
            InputTime::JustReleased {..} => "just_released".to_string(),
        }));
        fields.add_field_method_get("kind", |_, this| Ok(Some(match &this.data {
            InputData::Analog {..} => "analog".to_string(),
            InputData::Binary {..} => "binary".to_string(),
            InputData::Directional {..} => "directional".to_string(),
            InputData::VecChain {..} => "vec_chain".to_string(),
        })));
        fields.add_field_method_get("power", |_, this| Ok(this.power()));
        fields.add_field_method_get("time_held", |_, this| match this.time {
            InputTime::Held { secs_start,.. } => Ok(Some(secs_start)),
            _ => Ok(None),
        });
        fields.add_field_method_get("time", |_, this| match this.time {
            InputTime::NotPressed => Ok(None),
            InputTime::JustPressed { secs } => Ok(Some(secs)),
            InputTime::Held { secs, .. } => Ok(Some(secs)),
            InputTime::JustReleased { secs } => Ok(Some(secs)),
        });
        fields.add_field_method_get("vec_chain", |lua, this| match &this.data {
            InputData::VecChain { vecs } => {
                let mut chain = Vec::new();
                for vec in vecs.iter() {
                    let table = lua.create_table()?;
                    table.set("time", vec.time)?;
                    table.set("pos",  LuaVec2::new(vec.value))?;
                    chain.push(table);
                }
                Ok(Some(chain))
            },
            _ => Ok(None),
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_method("is_blank", |_, this, ()| Ok(this.is_blank()));
        methods.add_method("time_scaled_power", |_, this, ()| Ok(this.time_scaled_power()));
    }
}

#[derive(Clone, Copy, Debug, Default, InspectorOptions, PartialEq)]
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

#[derive(Clone, Component, Debug, Default, InspectorOptions, PartialEq)]
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
impl LuaUserData for ActionState {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("gamepad", |_, this| Ok(this.gamepad_id));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
        methods.add_meta_method(LuaMetaMethod::Index, |_, this, action: String| {
            Ok(this.inputs.get(&action).cloned())
        });
    }
}
impl LuaMod for ActionState {
    fn mod_name() -> &'static str { "Action" }
    fn register_defs(_lua: &Lua, _table: &mut LuaTable) -> Result<(), mlua::Error> {
        Ok(())
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
        actions.insert("run".into(),        vec![InputCode::Key(KeyCode::LShift)]);
        actions.insert("cam_switch".into(), vec![InputCode::Key(KeyCode::C), InputCode::MouseButton(MouseButton::Middle)]);
        actions.insert("select".into(),     vec![InputCode::Key(KeyCode::E),  InputCode::MouseButton(MouseButton::Left)]);
        actions.insert("cam_drag".into(),   vec![InputCode::MouseMotion { hold: MouseButton::Right, sensitivity: 1. }]);
        actions.insert("zoom_in".into(),    vec![InputCode::Key(KeyCode::Equals), InputCode::Scroll(ScrollWheel::Up)]);
        actions.insert("zoom_out".into(),   vec![InputCode::Key(KeyCode::Minus),  InputCode::Scroll(ScrollWheel::Down)]);
        InputMap {
            actions,
            cam_speed: 6.,
            zoom_speed: 12.,
        }
    }
}