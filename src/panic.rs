//! Default panic handler for `no_std` patch binaries.
//!
//! Enabled by the `panic-halt` feature (on by default). Loops forever
//! on panic — the firmware will detect the stall via frame drops.

#[cfg(all(feature = "panic-halt", not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
