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
/// instance. It is evaluated once, on the first call from the firmware,
/// whichever entry point that turns out to be.
#[macro_export]
macro_rules! endless_patch {
    ($PatchType:ty, $init_expr:expr) => {
        // ── singleton storage ────────────────────────────────────────
        //
        // Uses `UnsafeCell` with a manual `Sync` impl instead of
        // `static mut` (which is deprecated in edition 2024).

        struct PatchState {
            instance: core::mem::MaybeUninit<$PatchType>,
            initialized: bool,
            error: bool,
        }

        /// Wrapper to make `PatchState` usable in a `static`.
        #[repr(transparent)]
        struct SyncPatchState(core::cell::UnsafeCell<PatchState>);

        // SAFETY: bare-metal single-core. The firmware never calls two
        // entry points concurrently, so no two references to the state
        // are ever live at once.
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

        /// Get the patch instance, constructing it on first use.
        ///
        /// Every entry point goes through here, so the instance is valid no
        /// matter which function the firmware calls first and whether or not
        /// `patch_init` accepted the ABI version.
        ///
        /// # Safety
        /// Caller must ensure no concurrent access (guaranteed on bare-metal).
        #[inline(always)]
        unsafe fn patch() -> &'static mut $PatchType {
            let s = unsafe { state() };
            if !s.initialized {
                s.instance.write($init_expr);
                s.initialized = true;
            }
            unsafe { s.instance.assume_init_mut() }
        }

        /// Metadata for a raw parameter index, or the defaults when the index
        /// is out of range.
        ///
        /// # Safety
        /// Caller must ensure no concurrent access (guaranteed on bare-metal).
        #[inline(always)]
        unsafe fn param_metadata(idx: i32) -> $crate::ParameterMetadata {
            match $crate::ParamId::from_raw(idx) {
                Some(param) => $crate::Patch::param_metadata(unsafe { patch() }, param),
                None => $crate::ParameterMetadata::default(),
            }
        }

        // ── ABI functions (extern "C") ───────────────────────────────
        //
        // These are referenced by the header below through `sym` operands,
        // so they need no `no_mangle` and cannot collide with symbols in
        // the patch crate.

        unsafe extern "C" fn patch_init(env: *const $crate::abi::PatchEnv) {
            let supported =
                !env.is_null() && unsafe { (*env).abi_version } == u32::from($crate::abi::PATCH_ABI_VERSION);
            if !supported {
                unsafe { state().error = true };
                return;
            }
            $crate::Patch::init(unsafe { patch() });
        }

        unsafe extern "C" fn patch_agent_update_buffers(
            _env: *const $crate::abi::PatchEnv,
            left: *mut f32,
            right: *mut f32,
            samples: i32,
        ) -> f32 {
            if left.is_null() || right.is_null() {
                return 0.0;
            }
            // A negative sample count is a firmware bug; treat it as empty
            // rather than building a slice of absurd length.
            let n = usize::try_from(samples).unwrap_or(0);
            let left = unsafe { core::slice::from_raw_parts_mut(left, n) };
            let right = unsafe { core::slice::from_raw_parts_mut(right, n) };
            $crate::Patch::process_audio(unsafe { patch() }, left, right)
        }

        unsafe extern "C" fn patch_agent_set_buffer(_env: *const $crate::abi::PatchEnv, buffer: *mut f32) {
            if buffer.is_null() {
                unsafe { state().error = true };
                return;
            }
            let buf = unsafe { core::slice::from_raw_parts_mut(buffer, $crate::WORKING_BUFFER_SIZE) };
            $crate::Patch::set_working_buffer(unsafe { patch() }, buf);
        }

        unsafe extern "C" fn patch_agent_get_buffer_size(_env: *const $crate::abi::PatchEnv) -> i32 {
            $crate::WORKING_BUFFER_SIZE as i32
        }

        unsafe extern "C" fn patch_agent_get_param_min(_env: *const $crate::abi::PatchEnv, idx: i32) -> f32 {
            unsafe { param_metadata(idx) }.min
        }

        unsafe extern "C" fn patch_agent_get_param_max(_env: *const $crate::abi::PatchEnv, idx: i32) -> f32 {
            unsafe { param_metadata(idx) }.max
        }

        unsafe extern "C" fn patch_agent_get_param_default(_env: *const $crate::abi::PatchEnv, idx: i32) -> f32 {
            unsafe { param_metadata(idx) }.default
        }

        unsafe extern "C" fn patch_agent_is_param_enabled(
            _env: *const $crate::abi::PatchEnv,
            idx: i32,
            source_id: i32,
        ) -> u8 {
            let is_knob = source_id == 0;
            u8::from(is_knob && $crate::ParamId::from_raw(idx).is_some())
        }

        unsafe extern "C" fn patch_agent_get_param_name(
            _env: *const $crate::abi::PatchEnv,
            _idx: i32,
            out: *mut u8,
            buf_size: usize,
        ) {
            if !out.is_null() && buf_size > 0 {
                unsafe { *out = 0 };
            }
        }

        unsafe extern "C" fn patch_agent_get_param_unit(
            _env: *const $crate::abi::PatchEnv,
            _idx: i32,
            out: *mut u8,
            buf_size: usize,
        ) {
            if !out.is_null() && buf_size > 0 {
                unsafe { *out = 0 };
            }
        }

        unsafe extern "C" fn patch_agent_set_param(_env: *const $crate::abi::PatchEnv, idx: i32, value: f32) {
            if let Some(param) = $crate::ParamId::from_raw(idx) {
                $crate::Patch::set_param(unsafe { patch() }, param, value);
            }
        }

        unsafe extern "C" fn patch_agent_special_action(_env: *const $crate::abi::PatchEnv, idx: i32) {
            if let Some(action) = $crate::ActionId::from_raw(idx) {
                $crate::Patch::handle_action(unsafe { patch() }, action);
            }
        }

        unsafe extern "C" fn patch_agent_get_state_idx(_env: *const $crate::abi::PatchEnv) -> i32 {
            if unsafe { state().error } {
                return $crate::Color::Red as i32;
            }
            $crate::Patch::state_led_color(unsafe { patch() }) as i32
        }

        // ── PatchHeader via global_asm! ──────────────────────────────
        //
        // Emitted in assembly so linker-computed values (image_size,
        // bss_begin, bss_size) can be filled in at link time. The magic
        // and ABI version come from `crate::abi` through `const` operands,
        // and the function addresses from `sym` operands, so the header
        // cannot drift from the constants `patch_init` checks against.
        // The linker sets the Thumb bit on function addresses for Thumb
        // targets; `just check-example` verifies the emitted bytes.

        core::arch::global_asm!(
            ".section .patch_header, \"a\"",
            ".balign 4",
            ".global patch_header",
            "patch_header:",
            // magic
            ".word {magic}",
            // abi_version (u16) + flags (u16)
            ".short {abi_version}",
            ".short 0",
            // init
            ".word {init}",
            // agent function pointers
            ".word {update_buffers}",
            ".word {set_buffer}",
            ".word {get_buffer_size}",
            ".word {get_param_min}",
            ".word {get_param_max}",
            ".word {get_param_default}",
            ".word {is_param_enabled}",
            ".word {get_param_name}",
            ".word {get_param_unit}",
            ".word {set_param}",
            ".word {special_action}",
            ".word {get_state_idx}",
            // image_size (linker-computed absolute symbol)
            ".word patch_image_size",
            // bss_begin
            ".word __patch_bss_start",
            // bss_size (linker-computed absolute symbol)
            ".word patch_bss_size",
            // reserved[16]
            ".fill 16, 4, 0",
            magic = const $crate::abi::PATCH_MAGIC,
            abi_version = const $crate::abi::PATCH_ABI_VERSION,
            init = sym patch_init,
            update_buffers = sym patch_agent_update_buffers,
            set_buffer = sym patch_agent_set_buffer,
            get_buffer_size = sym patch_agent_get_buffer_size,
            get_param_min = sym patch_agent_get_param_min,
            get_param_max = sym patch_agent_get_param_max,
            get_param_default = sym patch_agent_get_param_default,
            is_param_enabled = sym patch_agent_is_param_enabled,
            get_param_name = sym patch_agent_get_param_name,
            get_param_unit = sym patch_agent_get_param_unit,
            set_param = sym patch_agent_set_param,
            special_action = sym patch_agent_special_action,
            get_state_idx = sym patch_agent_get_state_idx,
        );
    };
}
