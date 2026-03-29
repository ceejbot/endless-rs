//! The `endless_patch!` macro — generates ABI glue and the patch header.

/// Register a [`Patch`](crate::Patch) implementation as the active patch.
///
/// Generates all `extern "C"` ABI functions and emits the `PatchHeader`
/// in the `.patch_header` linker section via `global_asm!`.
///
/// # Usage
///
/// ```ignore
/// endless_patch!(MyPatch, MyPatch { field: value });
/// ```
///
/// The first argument is the concrete type implementing `Patch`.
/// The second argument is the expression used to construct the initial
/// instance.
#[macro_export]
macro_rules! endless_patch {
    ($PatchType:ty, $init_expr:expr) => {
        // ── singleton storage ────────────────────────────────────────
        //
        // Uses `UnsafeCell` with a manual `Sync` impl instead of
        // `static mut` (which is deprecated in edition 2024).
        // Safety: bare-metal single-core — the firmware guarantees no
        // concurrent access to these functions.

        struct PatchState {
            instance: core::mem::MaybeUninit<$PatchType>,
            initialized: bool,
            error: bool,
        }

        /// Wrapper to make `PatchState` usable in a `static`.
        ///
        /// SAFETY: bare-metal single-core — no concurrent access.
        #[repr(transparent)]
        struct SyncPatchState(core::cell::UnsafeCell<PatchState>);
        unsafe impl Sync for SyncPatchState {}

        static PATCH_STATE: SyncPatchState = SyncPatchState(core::cell::UnsafeCell::new(PatchState {
            instance: core::mem::MaybeUninit::uninit(),
            initialized: false,
            error: false,
        }));

        /// Get a mutable reference to the patch state.
        ///
        /// # Safety
        /// Caller must ensure no concurrent access (guaranteed on bare-metal).
        #[inline(always)]
        unsafe fn state() -> &'static mut PatchState {
            unsafe { &mut *PATCH_STATE.0.get() }
        }

        /// Get a mutable reference to the initialized patch instance.
        ///
        /// # Safety
        /// Must only be called after `patch_init` has succeeded.
        #[inline(always)]
        unsafe fn patch() -> &'static mut $PatchType {
            unsafe { state().instance.assume_init_mut() }
        }

        /// Get a shared reference to the initialized patch instance.
        ///
        /// # Safety
        /// Must only be called after `patch_init` has succeeded.
        #[inline(always)]
        unsafe fn patch_ref() -> &'static $PatchType {
            unsafe { state().instance.assume_init_ref() }
        }

        // ── ABI functions (extern "C") ───────────────────────────────

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_init(env: *const $crate::abi::PatchEnv) {
            let env = unsafe { &*env };
            if env.abi_version != $crate::abi::PATCH_ABI_VERSION as u32 {
                unsafe { state().error = true };
                return;
            }
            let s = unsafe { state() };
            if !s.initialized {
                s.instance.write($init_expr);
                s.initialized = true;
            }
            $crate::Patch::init(unsafe { patch() });
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_update_buffers(
            _env: *const $crate::abi::PatchEnv,
            left: *mut f32,
            right: *mut f32,
            samples: i32,
        ) -> f32 {
            let n = samples as usize;
            let left = unsafe { core::slice::from_raw_parts_mut(left, n) };
            let right = unsafe { core::slice::from_raw_parts_mut(right, n) };
            $crate::Patch::process_audio(unsafe { patch() }, left, right)
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_set_buffer(_env: *const $crate::abi::PatchEnv, buffer: *mut f32) {
            if buffer.is_null() {
                unsafe { state().error = true };
                return;
            }
            let buf = unsafe { core::slice::from_raw_parts_mut(buffer, $crate::WORKING_BUFFER_SIZE) };
            $crate::Patch::set_working_buffer(unsafe { patch() }, buf);
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_get_buffer_size(_env: *const $crate::abi::PatchEnv) -> i32 {
            $crate::WORKING_BUFFER_SIZE as i32
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_get_param_min(_env: *const $crate::abi::PatchEnv, idx: i32) -> f32 {
            let param = match $crate::ParamId::from_raw(idx) {
                Some(p) => p,
                None => return 0.0,
            };
            $crate::Patch::param_metadata(unsafe { patch_ref() }, param).min
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_get_param_max(_env: *const $crate::abi::PatchEnv, idx: i32) -> f32 {
            let param = match $crate::ParamId::from_raw(idx) {
                Some(p) => p,
                None => return 1.0,
            };
            $crate::Patch::param_metadata(unsafe { patch_ref() }, param).max
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_get_param_default(_env: *const $crate::abi::PatchEnv, idx: i32) -> f32 {
            let param = match $crate::ParamId::from_raw(idx) {
                Some(p) => p,
                None => return 0.5,
            };
            $crate::Patch::param_metadata(unsafe { patch_ref() }, param).default
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_is_param_enabled(
            _env: *const $crate::abi::PatchEnv,
            idx: i32,
            source_id: i32,
        ) -> u8 {
            let is_knob = source_id == 0;
            if is_knob && idx >= 0 && (idx as usize) < $crate::NUM_PARAMS {
                1
            } else {
                0
            }
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_get_param_name(
            _env: *const $crate::abi::PatchEnv,
            _idx: i32,
            out: *mut u8,
            buf_size: usize,
        ) {
            if buf_size > 0 {
                unsafe { *out = 0 };
            }
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_get_param_unit(
            _env: *const $crate::abi::PatchEnv,
            _idx: i32,
            out: *mut u8,
            _buf_size: usize,
        ) {
            unsafe { *out = 0 };
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_set_param(_env: *const $crate::abi::PatchEnv, idx: i32, value: f32) {
            if let Some(param) = $crate::ParamId::from_raw(idx) {
                $crate::Patch::set_param(unsafe { patch() }, param, value);
            }
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_special_action(_env: *const $crate::abi::PatchEnv, idx: i32) {
            if let Some(action) = $crate::ActionId::from_raw(idx) {
                $crate::Patch::handle_action(unsafe { patch() }, action);
            }
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn patch_agent_get_state_idx(_env: *const $crate::abi::PatchEnv) -> i32 {
            if unsafe { state().error } {
                return $crate::Color::Red as i32;
            }
            $crate::Patch::state_led_color(unsafe { patch_ref() }) as i32
        }

        // ── PatchHeader via global_asm! ──────────────────────────────
        //
        // Emitted in assembly so linker-computed values (image_size,
        // bss_begin, bss_size) can be filled in at link time.
        // The assembler automatically sets the Thumb bit on function
        // addresses for Thumb targets.

        core::arch::global_asm!(
            ".section .patch_header, \"a\"",
            ".balign 4",
            ".global patch_header",
            "patch_header:",
            // magic
            ".word 0x48435450",
            // abi_version (u16) + flags (u16)
            ".short 0x000B",
            ".short 0x0000",
            // init
            ".word patch_init",
            // agent function pointers
            ".word patch_agent_update_buffers",
            ".word patch_agent_set_buffer",
            ".word patch_agent_get_buffer_size",
            ".word patch_agent_get_param_min",
            ".word patch_agent_get_param_max",
            ".word patch_agent_get_param_default",
            ".word patch_agent_is_param_enabled",
            ".word patch_agent_get_param_name",
            ".word patch_agent_get_param_unit",
            ".word patch_agent_set_param",
            ".word patch_agent_special_action",
            ".word patch_agent_get_state_idx",
            // image_size (linker-computed absolute symbol)
            ".word patch_image_size",
            // bss_begin
            ".word __patch_bss_start",
            // bss_size (linker-computed absolute symbol)
            ".word patch_bss_size",
            // reserved[16]
            ".fill 16, 4, 0",
        );
    };
}
