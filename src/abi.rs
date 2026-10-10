//! C ABI types matching `PatchABI.h` from the FxPatchSDK.
//!
//! These `#[repr(C)]` structs define the binary interface between the
//! Endless firmware loader and a patch image.

use core::ffi::c_void;

/// Magic value: ASCII `'PTCH'` stored little-endian.
pub const PATCH_MAGIC: u32 = 0x4843_5450;

/// ABI version. Must match the firmware's expected version.
pub const PATCH_ABI_VERSION: u16 = 0x000B;

/// Size in bytes of the header the firmware reads from offset 0.
pub const PATCH_HEADER_SIZE: usize = 136;

/// Environment provided by the firmware to the patch at init/update time.
#[repr(C)]
pub struct PatchEnv {
    pub abi_version: u32,
    pub user_ctx: *mut c_void,
}

/// Header placed at offset 0 of the patch binary in RAM.
///
/// In practice this is emitted via `global_asm!` (not as a Rust static)
/// so that linker-computed fields (`image_size`, `bss_begin`, `bss_size`)
/// can be filled in at link time. This struct exists as a reference for
/// the binary layout; it is not instantiated directly. The size assertion
/// below ties it to [`PATCH_HEADER_SIZE`], and `just check-example`
/// verifies the bytes the macro actually emits.
#[repr(C)]
pub struct PatchHeader {
    pub magic: u32,
    pub abi_version: u16,
    pub flags: u16,

    pub init: usize,

    pub agent_update_buffers: Option<unsafe extern "C" fn(*const PatchEnv, *mut f32, *mut f32, i32) -> f32>,
    pub agent_set_buffer: Option<unsafe extern "C" fn(*const PatchEnv, *mut f32)>,
    pub agent_get_buffer_size: Option<unsafe extern "C" fn(*const PatchEnv) -> i32>,
    pub agent_get_param_min: Option<unsafe extern "C" fn(*const PatchEnv, i32) -> f32>,
    pub agent_get_param_max: Option<unsafe extern "C" fn(*const PatchEnv, i32) -> f32>,
    pub agent_get_param_default: Option<unsafe extern "C" fn(*const PatchEnv, i32) -> f32>,
    pub agent_is_param_enabled: Option<unsafe extern "C" fn(*const PatchEnv, i32, i32) -> u8>,
    pub agent_get_param_name: Option<unsafe extern "C" fn(*const PatchEnv, i32, *mut u8, usize)>,
    pub agent_get_param_unit: Option<unsafe extern "C" fn(*const PatchEnv, i32, *mut u8, usize)>,
    pub agent_set_param: Option<unsafe extern "C" fn(*const PatchEnv, i32, f32)>,
    pub agent_special_action: Option<unsafe extern "C" fn(*const PatchEnv, i32)>,
    pub agent_get_state_idx: Option<unsafe extern "C" fn(*const PatchEnv) -> i32>,

    pub image_size: u32,
    pub bss_begin: usize,
    pub bss_size: u32,
    pub reserved: [u32; 16],
}

// The layout only matches the firmware on the 32-bit target; on a 64-bit host
// the pointer-sized fields double, so the check is confined to the target.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<PatchHeader>() == PATCH_HEADER_SIZE);
    assert!(core::mem::size_of::<PatchEnv>() == 8);
};
