//!
//! @file errors.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Macros for reporting fatal errors.
//! @bug No known bugs.
//!

use std::ffi::{c_ulong, c_char};
use std::panic::PanicInfo;

pub use ctypes::cstr;

extern "system" {
    /// SKSE panic function.
    #[link_name = "SKSE64_Errors__assert_failed__"]
    pub fn skse_halt_impl(file: *const c_char, line: c_ulong, msg: *const c_char) -> !;

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

/// Uses the SKSE panic handler to terminate the application.
#[macro_export]
macro_rules! skse_halt {
    ( $s:expr ) => {{
        let s = $crate::errors::cstr!($s);
        let file = $crate::errors::cstr!($crate::core::file!());
        let line = $crate::core::line!();

        unsafe {
            $crate::errors::skse_halt_impl(file, line as $crate::core::ffi::c_ulong, s);
        }
    }};
}
pub use skse_halt;

/// Uses the SKSE panic handler to assert a condition.
#[macro_export]
macro_rules! skse_assert {
    ( $cond:expr ) => {
        if !($cond) {
            $crate::skse_halt!($crate::core::stringify!($cond));
        }
    };

    ( $cond:expr, $lit:expr ) => {
        if !($cond) {
            $crate::skse_halt!($lit);
        }
    };
}
pub use skse_assert;

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
