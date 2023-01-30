//!
//! @file log.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Wraps the SKSE logging API.
//! @bug No known bugs.
//!

use std::ffi::{c_char, CString};

extern "system" {
    #[link_name = "SKSE64_DebugLog__open__"]
    fn glog_open(path: *const c_char);

    #[link_name = "SKSE64_DebugLog__message__"]
    fn glog_message(msg: *const c_char);

    #[link_name = "SKSE64_DebugLog__error__"]
    fn glog_error(msg: *const c_char);
}

/// Opens a log file with the given name in the SKSE log directory.
pub (in crate) fn open(
    log: String
) {
    unsafe {
        // SAFETY: We are giving this function a valid C string.
        glog_open(CString::new(log).unwrap().as_c_str().as_ptr());
    }
}

#[doc(hidden)]
pub fn skse_message_impl(
    msg: String
) {
    unsafe {
        // SAFETY: we are giving this fn a valid string.
        glog_message(CString::new(msg).unwrap().as_c_str().as_ptr());
    }
}

#[doc(hidden)]
pub fn skse_error_impl(
    msg: String
) {
    unsafe {
        // SAFETY: We are giving this fn a valid string.
        glog_error(CString::new(msg).unwrap().as_c_str().as_ptr());
    }
}

#[macro_export]
macro_rules! skse_message {
    ( $($fmt:tt)* ) => {
        $crate::log::skse_message_impl(::std::format!($($fmt)*));
    };
}

#[macro_export]
macro_rules! skse_error {
    ( $($fmt:tt)* ) => {
        $crate::log::skse_error_impl(::std::format!($($fmt)*));
    };
}

pub use skse_message;
pub use skse_error;
