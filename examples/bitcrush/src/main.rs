#![no_std]
#![no_main]

use endless_rs::{endless_patch, ActionId, Color, ParamId, ParameterMetadata, Patch};

struct Bitcrush {
    state: bool,
    param_value: f32,
}

impl Patch for Bitcrush {
    fn process_audio(&mut self, left: &mut [f32], right: &mut [f32]) -> f32 {
        let values = libm::powf(2.0, (1.0 - self.param_value) * 4.0 + 5.0);
        let values_inv = 1.0 / values;

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = libm::roundf(*l * values) * values_inv;
            *r = libm::roundf(*r * values) * values_inv;
        }
        0.0
    }

    fn param_metadata(&self, _param: ParamId) -> ParameterMetadata {
        ParameterMetadata {
            min: 0.0,
            max: 1.0,
            default: 0.5,
        }
    }

    fn set_param(&mut self, param: ParamId, value: f32) {
        if param == ParamId::Left {
            self.param_value = value;
        }
    }

    fn handle_action(&mut self, _action: ActionId) {
        self.state = !self.state;
    }

    fn state_led_color(&self) -> Color {
        if self.state {
            Color::Blue
        } else {
            Color::DimBlue
        }
    }
}

endless_patch!(Bitcrush, Bitcrush {
    state: false,
    param_value: 0.5,
});
