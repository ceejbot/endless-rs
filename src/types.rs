//! Public types for the Endless patch API.

/// Audio sample rate in Hz.
pub const SAMPLE_RATE: u32 = 48_000;

/// Size of the external working buffer (in floats).
pub const WORKING_BUFFER_SIZE: usize = 2_400_000;

/// Number of hardware knob parameters.
pub const NUM_PARAMS: usize = 3;

/// LED color indices matching `Patch::Color` in the C++ SDK.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    DimWhite = 0,
    DarkRed = 1,
    DarkLime = 2,
    DarkCobalt = 3,
    LightYellow = 4,
    DimBlue = 5,
    Beige = 6,
    DimCyan = 7,
    Magenta = 8,
    LightBlue = 9,
    PastelGreen = 10,
    DimYellow = 11,
    Blue = 12,
    LightGreen = 13,
    Red = 14,
    DimGreen = 15,
}

/// Hardware knob parameter identifiers.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamId {
    Left = 0,
    Mid = 1,
    Right = 2,
}

impl ParamId {
    /// Convert from a raw ABI index. Returns `None` for out-of-range values.
    #[must_use]
    pub fn from_raw(idx: i32) -> Option<Self> {
        match idx {
            0 => Some(Self::Left),
            1 => Some(Self::Mid),
            2 => Some(Self::Right),
            _ => None,
        }
    }
}

/// Foot-switch action identifiers.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionId {
    LeftFootSwitchPress = 0,
    LeftFootSwitchHold = 1,
}

impl ActionId {
    /// Convert from a raw ABI index. Returns `None` for out-of-range values.
    #[must_use]
    pub fn from_raw(idx: i32) -> Option<Self> {
        match idx {
            0 => Some(Self::LeftFootSwitchPress),
            1 => Some(Self::LeftFootSwitchHold),
            _ => None,
        }
    }
}

/// Metadata describing a single parameter's range and default.
#[derive(Clone, Copy, Debug)]
pub struct ParameterMetadata {
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

impl Default for ParameterMetadata {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            default: 0.5,
        }
    }
}
