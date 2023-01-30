//!
//! @file errors.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Macros for reporting fatal errors.
//! @bug No known bugs.
//!

use core::ffi::{c_ulong, c_char};
pub use ctypes::cstr;

extern "system" {
    /// SKSE panic function.
    #[link_name = "SKSE64_Errors__assert_failed__"]
    pub fn skse_panic_impl(file: *const c_char, line: c_ulong, msg: *const c_char) -> !;
}

/// Uses the SKSE panic handler to terminate the application.
#[macro_export]
macro_rules! skse_halt {
    ( $s:expr ) => {{
        let s = $crate::errors::cstr!($s);
        let file = $crate::errors::cstr!($crate::core::file!());
        let line = $crate::core::line!();

        unsafe {
            $crate::errors::skse_panic_impl(file, line as $crate::core::ffi::c_ulong, s);
        }
    }};
}

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
pub use skse_halt;
