//!
//! @file errors.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Macros for reporting fatal errors.
//! @bug No known bugs.
//!

use std::panic::PanicInfo;

use crate::log::skse_fatal;

extern "system" {
    /// Halts the execution of a SKSE plugin.
    fn stop_plugin() -> !;
}

// Implement stop_plugin().
std::arch::global_asm! {
    include_str!("stop_plugin.S"),
    options(att_syntax)
}

///
/// Handles a Rust panic, redirecting the output to the skse_error!() macro.
///
/// Allocation in a panic handler is of the devil, so we don't do that here.
///
pub (in crate) fn skse_panic(
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
    unsafe { stop_plugin(); }
}
