use std::{collections::{hash_map::RawEntryMut, HashMap}, f32::consts::PI};

use bevy::{prelude::*, input::mouse::MouseWheel};
use bevy_inspector_egui::RegisterInspectable;

use crate::data::input::*;

#[derive(Clone, Debug, Default)]
pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_inspectable::<ActionState>()
            .add_system(update_action_state)
        ;
    }
}

pub fn update_action_state(
    time: Res<Time>,
    gamepad_input:      Res<Input<GamepadButton>>,
    mut gamepad_events: EventReader<GamepadEvent>,
    keyboard_input:     Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut query: Query<(&mut ActionState, &InputMap)>,
) {
    let mut scroll = 0.;
    let secs = time.seconds_since_startup();
    let mut gamepad_axes = HashMap::<Gamepad, HashMap<GamepadAxisType, f32>>::new();
    for (mut action_state, mappings) in query.iter_mut() {
        for (action, inputs) in mappings.actions.iter() {
            let mut max_ts = InputTS::default();
            for input in inputs {
                max_ts = max_ts.combine(match input {
                    InputCode::Key(code) => InputTS {
                        time: InputTime::from_input(&keyboard_input, *code, secs),
                        data: InputData::Binary { pressed: true }
                    },
                    InputCode::Mouse(code) => InputTS {
                        time: InputTime::from_input(&mouse_button_input, *code, secs),
                        data: InputData::Binary { pressed: true }
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
                            info!("scroll {} wheel {:?}", scroll, wheel);
                            InputTS {
                                time: InputTime::Held { secs },
                                data: InputData::Directional { state: scroll, angle: if scroll > 0. { PI * 0.5 } else { PI * 1.5} },
                            }
                        } else {
                            InputTS::default()
                        }
                    },
                    InputCode::Joystick { side, angle, sensitivity } => { todo!() },
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
                        if let Some(id) = action_state.gamepad_id {
                            let gamepad = Gamepad { id };
                            if let Some(axes) = gamepad_axes.get(&gamepad) {
                                if let Some(state) = axes.get(code) {
                                    InputTS {
                                        time: InputTime::Held { secs },
                                        data: InputData::Analog { state: *state },
                                    }
                                } else { InputTS::default() }
                            } else { InputTS::default() }
                        } else { InputTS::default() }
                        // let gamepad = Gamepad { id };

                        // InputTS {
                        //     time: InputTime::from_input(&gamepad_axis_input, GamepadAxis { gamepad, axis_type: *code }, secs),
                        //     data: InputData::Binary { pressed: true }
                        // }
                    } else { InputTS::default() },
                });
            }
            match action_state.inputs.raw_entry_mut().from_key(action) {
                RawEntryMut::Occupied(mut e) => {
                    e.insert(if max_ts.data.is_empty() {
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
}