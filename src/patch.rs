//! The `Patch` trait — the safe Rust API for Endless effects.

use crate::types::{ActionId, Color, ParamId, ParameterMetadata};

/// Trait implemented by Endless effect patches.
///
/// All methods have default (no-op) implementations so you only need to
/// override the ones your effect requires. At minimum you'll want
/// [`process_audio`](Patch::process_audio).
pub trait Patch {
    /// Called once at patch load time.
    fn init(&mut self) {}

    /// Called once to provide the external working buffer.
    ///
    /// This buffer lives in external RAM with higher access latency.
    /// The slice length is [`WORKING_BUFFER_SIZE`](crate::WORKING_BUFFER_SIZE).
    fn set_working_buffer(&mut self, _buffer: &'static mut [f32]) {}

    /// Process audio buffers in-place. Called repeatedly from the audio thread.
    ///
    /// `left` and `right` are guaranteed to have equal length. Keep output
    /// samples in the range `(-1.0, 1.0)` to avoid digital clipping.
    ///
    /// The return value is passed back to the firmware (typically `0.0`).
    fn process_audio(&mut self, _left: &mut [f32], _right: &mut [f32]) -> f32 {
        0.0
    }

    /// Return metadata for the given parameter.
    fn param_metadata(&self, _param: ParamId) -> ParameterMetadata {
        ParameterMetadata::default()
    }

    /// Called when a knob value changes. Runs on the audio thread.
    fn set_param(&mut self, _param: ParamId, _value: f32) {}

    /// Called when a foot-switch action fires.
    fn handle_action(&mut self, _action: ActionId) {}

    /// Return the current LED color.
    fn state_led_color(&self) -> Color {
        Color::DimWhite
    }
}
