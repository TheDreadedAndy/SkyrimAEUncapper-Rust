//!
//! @file cstr.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief A macro for creating a C string literal.
//! @bug No known bugs.
//!

///
/// @brief Converts the given string literal into a C string.
///
#[macro_export]
macro_rules! cstr {
    ( $cstr:expr ) => {
        $crate::core::concat!($cstr, "\0").as_bytes().as_ptr() as *const $crate::core::ffi::c_char
    };
}
