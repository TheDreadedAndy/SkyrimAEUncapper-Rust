//!
//! @file errors.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Panic implementations for SKSE plugins.
//! @bug No known bugs.
//!

use core::panic::PanicInfo;

use core_util::RacyCell;

use crate::log::skse_fatal;

extern "system" {
    /// Halts the loading of a SKSE plugin.
    fn stop_plugin() -> !;
}

// Implement stop_plugin().
core::arch::global_asm! {
    include_str!("stop_plugin.S"),
    options(att_syntax)
}

pub (in crate) static SKSE_LOADER_DONE: RacyCell<bool> = RacyCell::new(false);

/// Stops the loading of the plugin when called during the load phase.
#[panic_handler]
fn skse_panic(
    info: &PanicInfo<'_>
) -> ! {
    skse_fatal!("{}", info);
    unsafe {
        // After loading has finished, it's not safe to halt the plugin by throwing an exception.
        // This is due to a bug in SKSE where exceptions are caught and then ignored if they
        // are thrown during the messaging phases. As such, we abort on panic if it happens after
        // the loading phase has finished.
        if *SKSE_LOADER_DONE.get() {
            libc::abort();
        } else {
            stop_plugin();
        }
    }
}
