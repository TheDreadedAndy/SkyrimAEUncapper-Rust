//!
//! @file errors.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Panic implementations for SKSE plugins.
//! @bug No known bugs.
//!

use std::panic::PanicInfo;

use crate::log::skse_fatal;

extern "system" {
    /// Halts the loading of a SKSE plugin.
    fn stop_plugin() -> !;
}

// Implement stop_plugin().
std::arch::global_asm! {
    include_str!("stop_plugin.S"),
    options(att_syntax)
}

/// Stops the loading of the plugin when called during the load phase.
pub (in crate) fn skse_loader_panic(
    info: &PanicInfo<'_>
) {
    skse_panic_print(info);
    unsafe { stop_plugin(); }
}

/// Terminates skyrim when the plugin encounters a fatal runtime error.
pub (in crate) fn skse_runtime_panic(
    info: &PanicInfo<'_>
) {
    skse_panic_print(info);
    std::process::abort();
}

///
/// Prints a Rust panic, redirecting the output to the skse_fatal!() macro.
///
/// Allocation in a panic handler is of the devil, so we don't do that here.
///
fn skse_panic_print(
    info: &PanicInfo<'_>
) {
    let (file, line) = info.location().map(|l| (l.file(), l.line())).unwrap_or(
        ("<Unknown location>", 0)
    );

    let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
        *s
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.as_ref()
    } else {
        "<Unknown error>"
    };

    skse_fatal!("{}:{}: `{}'", file, line, msg);
}
