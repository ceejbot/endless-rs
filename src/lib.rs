#![no_std]

#[doc(hidden)]
pub mod abi;
#[macro_use]
mod macros;
mod panic;
mod patch;
mod types;

pub use patch::Patch;
pub use types::{ActionId, Color, NUM_PARAMS, ParamId, ParameterMetadata, SAMPLE_RATE, WORKING_BUFFER_SIZE};
