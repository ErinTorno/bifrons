use std::{collections::{hash_map::RawEntryMut, HashMap}, f32::consts::PI};

use bevy::{prelude::*, input::mouse::{MouseWheel, MouseMotion}, window::CursorGrabMode};
use bevy_inspector_egui::RegisterInspectable;

use crate::{data::input::*, util::Timestamped};

#[derive(Clone, Debug, Default)]
pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system(update_action_state)
            .register_inspectable::<ActionState>()
            .register_inspectable::<InputData>()
            .register_inspectable::<InputTime>()
            .register_inspectable::<InputTS>()
        ;
    }
}

pub fn update_action_state(
    time: Res<Time>,
    gamepad_input:      Res<Input<GamepadButton>>,
    mut gamepad_events: EventReader<GamepadEvent>,
    keyboard_input:     Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut windows:        ResMut<Windows>,
    mut ev_scroll:      EventReader<MouseWheel>,
    mut mouse_motion:   EventReader<MouseMotion>,
    mut query:          Query<(&mut ActionState, &InputMap)>,
) {
    let window = windows.primary_mut();
    let mut grab_mode = CursorGrabMode::None;
    let mut scroll = 0.;
    let secs = time.elapsed_seconds_f64();
    let mut gamepad_axes = HashMap::<Gamepad, HashMap<GamepadAxisType, f32>>::new();
    let mut vec_chain = Vec::new();
    for event in mouse_motion.iter() {
        vec_chain.push(Timestamped { time: secs, value: event.delta });
    }
    for (mut action_state, mappings) in query.iter_mut() {
        for (action, inputs) in mappings.actions.iter() {
            let mut max_ts = InputTS::default();
            for input in inputs {
                max_ts = max_ts.combine(match input {
                    InputCode::Key(code) => InputTS {
                        time: InputTime::from_input(&keyboard_input, *code, secs),
                        data: InputData::Binary { pressed: true }
                    },
                    InputCode::MouseButton(code) => InputTS {
                        time: InputTime::from_input(&mouse_button_input, *code, secs),
                        data: InputData::Binary { pressed: true }
                    },
                    InputCode::MouseMotion { sensitivity, hold } => {
                        if vec_chain.is_empty() {
                            InputTS::default()
                        } else {
                            let t = InputTime::from_input(&mouse_button_input, *hold, secs);
                            if t != InputTime::NotPressed {
                                grab_mode = CursorGrabMode::Locked;
                                let mut vecs = if action_state.input_ts(action).is_vec_chain() {
                                    let ts = action_state.inputs.remove(action).unwrap();
                                    match ts.data {
                                        InputData::VecChain { vecs } => vecs,
                                        _ => unreachable!(),
                                    }
                                } else { Vec::new() };
                                let secs_start = vecs.get(0).map(|ts| ts.time).unwrap_or(secs);

                                for v in vec_chain.iter() {
                                    vecs.push(Timestamped { time: v.time, value: v.value * *sensitivity });
                                }
                                InputTS {
                                    time: InputTime::Held { secs, secs_start },
                                    data: InputData::VecChain { vecs },
                                }
                            } else { InputTS::default() }
                        }
                    },
                    InputCode::Gamepad(code) => if let Some(id) = action_state.gamepad_id {
                        let gamepad = Gamepad { id };
                        InputTS {
                            time: InputTime::from_input(&gamepad_input, GamepadButton { gamepad, button_type: *code }, secs),
                            data: InputData::Binary { pressed: true }
                        }
                    } else { InputTS::default() },
                    InputCode::Scroll(wheel) => {
                        for ev in ev_scroll.iter() {
                            scroll += ev.y;
                        }
                        if (scroll > 0. && *wheel == ScrollWheel::Up) || (scroll < 0. && *wheel == ScrollWheel::Down) {
                            InputTS {
                                time: InputTime::Held { secs, secs_start: secs },
                                data: InputData::Directional { state: scroll.abs(), angle: if scroll > 0. { PI * 0.5 } else { PI * 1.5} },
                            }
                        } else {
                            InputTS::default()
                        }
                    },
                    InputCode::Joystick { .. } => { todo!() },
                    InputCode::Shoulder(code) => if let Some(id) = action_state.gamepad_id {
                        for event in gamepad_events.iter() {
                            match event.event_type {
                                GamepadEventType::AxisChanged(axis, state) => {
                                    let hm = gamepad_axes.entry(event.gamepad).or_insert_with(|| HashMap::new());
                                    hm.insert(axis, state);
                                },
                                _ => (),
                            }
                        }
                        let gamepad = Gamepad { id };
                        if let Some(axes) = gamepad_axes.get(&gamepad) {
                            if let Some(state) = axes.get(code) {
                                InputTS {
                                    time: InputTime::Held   { secs, secs_start: secs },
                                    data: InputData::Analog { state: *state },
                                }
                            } else { InputTS::default() }
                        } else { InputTS::default() }
                    } else { InputTS::default() },
                });
            }
            match action_state.inputs.raw_entry_mut().from_key(action) {
                RawEntryMut::Occupied(mut e) => {
                    e.insert(if max_ts.is_blank() {
                        max_ts
                    } else {
                        max_ts.combine(e.get().clone())
                    });
                },
                RawEntryMut::Vacant(e) => {
                    e.insert(action.clone(), max_ts);
                },
            }
        }
    }
    window.set_cursor_visibility(grab_mode == CursorGrabMode::None);
    window.set_cursor_grab_mode(grab_mode);
}