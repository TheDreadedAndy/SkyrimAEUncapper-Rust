//!
//! @file utilities.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes the get_runtime_dir() function.
//! @bug No known bugs.
//!

pub use std::ffi::{c_char, CStr, CString};
pub use std::path::PathBuf;

extern "system" {
    fn SKSE64_Utilities__get_runtime_dir__() -> *const c_char;
}

pub fn get_runtime_dir() -> PathBuf {
    PathBuf::from(unsafe {
        // SAFETY: We ensure that we don't call any other functions which touch the
        //         underlying std::string for the runtime dir until we have copied it.
        CStr::from_ptr(SKSE64_Utilities__get_runtime_dir__())
    }.to_owned().into_string().unwrap())
}
