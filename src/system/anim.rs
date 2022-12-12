use std::f32::consts::PI;

use bevy::{prelude::*};

use crate::data::geometry::{LightAnim, LightAnimState, Light};

#[derive(Clone, Debug, Default)]
pub struct AnimPlugin;

impl Plugin for AnimPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .register_type::<Light>()
            .register_type::<LightAnim>()
            .register_type::<LightAnimState>()
            .add_system(anim_lights)
        ;
    }
}

pub fn anim_lights(
    time: Res<Time>,
    mut query: Query<(&LightAnim, &LightAnimState, Option<&mut DirectionalLight>, Option<&mut PointLight>, Option<&mut SpotLight>)>,
) {
    for (anim, anim_state, directional, point, spotlight) in query.iter_mut() {
        let new_value = anim_state.base_value * match anim {
            LightAnim::Constant { mul } =>  *mul,
            LightAnim::Sin { period, amplitude, phase_shift } => (time.elapsed_seconds() / period * 2. * PI + phase_shift).sin() * amplitude + 1.
        };
        if let Some(mut light) = directional {
            light.illuminance = new_value;
        }
        if let Some(mut light) = point {
            light.intensity = new_value;
        }
        if let Some(mut light) = spotlight {
            light.intensity = new_value;
        }
    }
}