//!
//! @file errors.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Macros for reporting fatal errors.
//! @bug No known bugs.
//!

use std::panic::PanicInfo;

extern "system" {
    /// SKSE panic function for rust code.
    #[link_name = "SKSE64_Errors__rust_panic__"]
    fn skse_rust_halt_impl(
        file: *const u8,
        file_len: usize,
        line: usize,
        msg: *const u8,
        msg_len: usize
    ) -> !;
}

// Implement skse_rust_halt_impl().
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

    unsafe {
        // SAFETY: We have given the function valid pointers and lengths.
        skse_rust_halt_impl(
            file.as_bytes().as_ptr(),
            file.len(),
            line as usize,
            msg.as_bytes().as_ptr(),
            msg.len()
        );
    }
}
