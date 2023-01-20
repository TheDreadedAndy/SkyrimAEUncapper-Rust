//!
//! @file cstr.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief A macro for creating a C string literal.
//! @bug No known bugs.
//!

use crate::c_char;

///
/// @brief Converts the given string literal into a C string.
///
#[macro_export]
macro_rules! cstr {
    ( $cstr:expr ) => {
        $crate::core::concat!($cstr, "\0").as_bytes().as_ptr() as *const $crate::core::ffi::c_char
    };
}

///
/// @brief Converts a string literal into a C string array.
///
pub const fn cstr_array<const N: usize>(
    s: &str
) -> [c_char; N] {
    let mut ret: [c_char; N] = [0; N];

    let s = s.as_bytes();
    if s.len() >= N {
        panic!("Cannot fit string in C array!");
    }

    let mut i = 0;
    while i < s.len() {
        ret[i] = s[i] as i8;
        i += 1;
    }

    ret
}
