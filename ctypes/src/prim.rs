//!
//! @file prim.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief Primitive C type definitions
//! @bug No known bugs.
//!

// Defined by core.
pub use core::ffi::c_void;
pub use core::ffi::{c_char, c_schar, c_uchar};
pub use core::ffi::{c_short, c_ushort};
pub use core::ffi::{c_int, c_uint};
pub use core::ffi::{c_long, c_ulong};
pub use core::ffi::{c_longlong, c_ulonglong};
pub use core::ffi::{c_float, c_double};

// The general way to get max_align_t is to union all the biggest primitive types,
// which would be double, long double, a pointer, size_t, and a function pointer.
// Rust doesn't have "long double", so I use u128 instead.
#[allow(non_camel_case_types, dead_code)]
pub union max_align_t {
    u: usize,
    d: f64,
    ud: u128,
    p: *mut u8,
    f: Option<fn()>
}

#[allow(non_camel_case_types)] pub type c_size_t = usize;
#[allow(non_camel_case_types)] pub type c_ssize_t = isize;
#[allow(non_camel_case_types)] pub type c_ptrdiff_t = isize;

///
/// @brief Allows the definition of any number of abstract types, with layouts unknown to Rust.
///
/// Types declared here will automatically have repr(C) applied.
///
#[macro_export]
macro_rules! abstract_type {
    ( $( $(#[$meta:meta])* $scope:vis type $name:ident );+; ) => {
        $($(#[$meta])* #[repr(C)] $scope struct $name {
            // Stop construction - without this anyone can construct.
            _private: [u8; 0],

            // Prevent the compiler from marking as Send, Sync, or Unpin.
            _marker: $crate::core::marker::PhantomData<
                (*mut u8, $crate::core::marker::PhantomPinned)
            >
        })*
    };
}

pub use abstract_type;
