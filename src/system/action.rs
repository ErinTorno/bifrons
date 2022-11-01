use std::collections::hash_map::RawEntryMut;

use bevy::prelude::*;
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
    keyboard_input:     Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut query: Query<(&mut ActionState, &InputMap)>,
) {
    let secs = time.seconds_since_startup();
    for (mut action_state, mappings) in query.iter_mut() {
        for (action, inputs) in mappings.actions.iter() {
            let mut max_ts = InputTS::NotPressed;
            for input in inputs {
                max_ts = max_ts.max_priority(match input {
                    InputCode::Key(code) => InputTS::from_input(&keyboard_input, *code, secs),
                    InputCode::Mouse(code) => InputTS::from_input(&mouse_button_input, *code, secs),
                    InputCode::Gamepad(code) => if let Some(id) = action_state.gamepad_id {
                        let gamepad = Gamepad { id };
                        InputTS::from_input(&gamepad_input, GamepadButton { gamepad, button_type: *code }, secs)
                    } else { InputTS::NotPressed },
                    _ => InputTS::NotPressed,
                });
            }
            match action_state.inputs.raw_entry_mut().from_key(action) {
                RawEntryMut::Occupied(mut e) => {
                    e.insert(if max_ts == InputTS::NotPressed {
                        max_ts
                    } else {
                        max_ts.max_priority(*e.get())
                    });
                },
                RawEntryMut::Vacant(e) => {
                    e.insert(action.clone(), max_ts);
                },
            }
        }
    }
}