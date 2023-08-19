//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Keywords/blocks/types which should be in Rust, but aren't.
//!
//! This file declares a number of macros which provides features that Rust probably *should*
//! provide by default. This includes:
//!   - A method of scoping the question mark operator.
//!   - A method of initializing static arrays where the size of the array has no real meaning and
//!     thus can't easily be defined.
//!   - A method of declaring abstract types, where the internal data layout is unknown.
//!
//! Additionally, two types of cells are provided to ease the declaration of global variables. The
//! first is called Later<T>, and acts as a once_cell that simply panics on reinitialization. This
//! allows the implementation to use a simple atomic, rather than a mutex. The second is an unsafe
//! cell wrapper that declares the underlying object as sync, which allows global variables to be
//! used in a way similar to C, and is slightly safer than using static mut.
//!

#![no_std]

// For macros.
pub use core;

use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Macros
////////////////////////////////////////////////////////////////////////////////////////////////////

///
/// Scopes the question mark operator within each of its blocks.
///
/// This macro can either be used as:
///     attempt! {{ /* stuff */ }}
/// or:
///     attempt! {{ /* stuff */ } catch(e) { /* more stuff */ }},
/// where the catch block is a call to map_err() on the original try block.
///
#[macro_export]
macro_rules! attempt {
    ( $try:block ) => {
        (|| $try)()
    };

    ( $try:block catch($arg:ident) $catch:block ) => {
        $crate::core::result::Result::map_err((|| $try)(), |$arg| $catch)
    };
}

///
/// Allows for a dynamically sized initialization of an array, capturing its size
/// in the identifier specified in the array type.
///
#[macro_export]
macro_rules! disarray {
    // Size capturing.
    ( $(#[$meta:meta])* $scope:vis static $arr:ident: [$type:ty; $size:ident] = [
        $($items:expr),*
    ]; ) => {
        $scope const $size: usize = $crate::disarray!(@maybe_count $($items),*);
        $(#[$meta])* $scope static $arr: [$type; $size] = [ $($items),* ];
    };

    // Non-capturing.
    ( $(#[$meta:meta])* $scope:vis static $arr:ident: [$type:ty] = [
        $($items:expr),*
    ]; ) => {
        $(#[$meta])* $scope static $arr: [$type; $crate::disarray!(@maybe_count $($items),*)] = [
            $($items),*
        ];
    };

    // Empty array len angers the compiler (idk).
    ( @maybe_count ) => { 0 };
    ( @maybe_count $($items:expr),+ ) => { [ $($crate::disarray!(@count $items)),* ].len() };

    // Make sure items are const.
    ( @count $item:expr ) => { 0 };
}

///
/// Allows the definition of any number of abstract types, with layouts unknown to Rust.
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// C string FFI
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Creates a CStr literal from a string literal.
#[macro_export]
macro_rules! cstr {
    ( $str:literal ) => {
        $crate::core::ffi::CStr::from_bytes_until_nul(
            $crate::core::concat!($str, "\0").as_bytes()
        ).unwrap()
    };
}

/// Creates a wide CStr literal from a string literal.
#[macro_export]
macro_rules! wcstr {
    ( $str:literal ) => {{
        const SIZE: usize = $crate::get_utf16_len($str) + 1;
        $crate::WideStr::new(&$crate::create_utf16_string::<SIZE>($str))
    }};
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// A wide string struct, for C FFI. Must be NUL-terminated.
pub struct WideStr<'a>(&'a [u16]);

impl<'a> WideStr<'a> {
    pub const fn new(
        s: &'a [u16]
    ) -> Self {
        assert!(s[s.len() - 1] == 0);
        Self(s)
    }

    pub const fn as_ptr(
        &self
    ) -> *const u16 {
        self.0.as_ptr()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Converts a UTF-8 string to a statically sized array of UTF-16.
#[doc(hidden)]
pub const fn create_utf16_string<const DIM: usize>(
    s: &'static str
) -> [u16; DIM] {
    let b       = s.as_bytes();
    let mut ret = [0; DIM];

    let mut b_i : usize = 0;
    let mut w_i : usize = 0;
    while b_i < b.len() {
        if b[b_i] & 0x80 == 0 {
            ret[w_i] = b[b_i] as u16;
        } else if b[b_i] & 0xE0 == 0xC0 {
            assert!(b[b_i + 1] & 0xC0 == 0x80);
            ret[w_i] = (((b[b_i] & 0x1F) as u16) << 6) | ((b[b_i + 1] & 0x3F) as u16);
            b_i += 1;
        } else if b[b_i] & 0xF0 == 0xE0 {
            assert!(b[b_i + 1] & 0xC0 == 0x80);
            assert!(b[b_i + 2] & 0xC0 == 0x80);
            ret[w_i] = (((b[b_i] & 0x0F) as u16) << 12) | (((b[b_i + 1] & 0x3F) as u16) << 6)
                                                        | ((b[b_i + 2] & 0x3F) as u16);
            assert!(ret[w_i] & 0xF800 != 0xD8);
            b_i += 2;
        } else {
            assert!(b[b_i] & 0xF8 == 0xF0);
            assert!(b[b_i + 1] & 0xC0 == 0x80);
            assert!(b[b_i + 2] & 0xC0 == 0x80);
            assert!(b[b_i + 3] & 0xC0 == 0x80);
            ret[w_i] = 0xD800 | (((b[b_i] & 0x03) as u16) << 8)
                              | (((b[b_i + 1] & 0x3F) as u16) << 2)
                              | (((b[b_i + 2] & 0x30) as u16) >> 4);
            ret[w_i + 1] = 0xDC00 | (((b[b_i + 2] & 0x0F) as u16) << 6)
                                  | ((b[b_i + 3] & 0x3F) as u16);
            w_i += 1;
            b_i += 3;
        }

        w_i += 1;
        b_i += 1;
    }

    assert!(w_i == DIM - 1);
    return ret;
}

// Counts the number of UTF-16 code points in a UTF-8 string.
#[doc(hidden)]
pub const fn get_utf16_len(
    s: &'static str
) -> usize {
    let b = s.as_bytes();

    let mut code_points : usize = 0;
    let mut i           : usize = 0;
    while i < b.len() {
        if b[i] & 0x80 == 0 {
            code_points += 1;
        } else if b[i] & 0xE0 == 0xC0 {
            code_points += 1;
            i += 1;
            assert!(b[i] & 0xC0 == 0x80);
        } else if b[i] & 0xF0 == 0xE0 {
            code_points += 1;
            i += 1;
            assert!(b[i] & 0xC0 == 0x80);
            i += 1;
            assert!(b[i] & 0xC0 == 0x80);
        } else {
            assert!(b[i] & 0xF8 == 0xF0);
            code_points += 1;
            i += 1;
            assert!(b[i] & 0xC0 == 0x80);
            i += 1;
            assert!(b[i] & 0xC0 == 0x80);
            i += 1;
            assert!(b[i] & 0xC0 == 0x80);
        }
        i += 1;
    }

    return code_points;
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Cells
////////////////////////////////////////////////////////////////////////////////////////////////////

/// The core later structure. It is illegal to deref it before initialization.
pub struct Later<T> {
    is_init: AtomicBool,
    pl: UnsafeCell<MaybeUninit<T>>
}

/// An unsafe cell which implements Sync.
#[repr(transparent)]
pub struct RacyCell<T>(UnsafeCell<T>);

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T> Later<T> {
    /// Creates a new later structure.
    pub const fn new() -> Self {
        Self {
            is_init: AtomicBool::new(false),
            pl: UnsafeCell::new(MaybeUninit::uninit())
        }
    }

    /// Initializes a later structure.
    pub fn init(
        &self,
        pl: T
    ) {
        assert!(!self.is_init.swap(true, Ordering::Relaxed));
        // SAFETY: We have ensured that we are the only object initializing the data.
        unsafe { (*self.pl.get()).write(pl); }
    }

    /// Checks if the instance has been initialized.
    pub fn is_init(
        &self
    ) -> bool {
        self.is_init.load(Ordering::Relaxed)
    }
}

impl<T> Deref for Later<T> {
    type Target = T;
    fn deref(
        &self
    ) -> &Self::Target {
        assert!(self.is_init.load(Ordering::Relaxed));
        // SAFETY: We have ensured that the object is initialized.
        unsafe { (*self.pl.get()).assume_init_ref() }
    }
}

impl<T> Drop for Later<T> {
    fn drop(
        &mut self
    ) {
        if *self.is_init.get_mut() {
            // SAFETY: We have ensured that the object is initialized.
            unsafe { (*self.pl.get()).assume_init_drop(); }
        }
    }
}

// Later is sync if T is.
unsafe impl<T: Sync> Sync for Later<T> {}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T> RacyCell<T> {
    /// Creates a new racy cell.
    pub const fn new(
        pl: T
    ) -> Self {
        Self(UnsafeCell::new(pl))
    }

    /// Gets a pointer to the cells data.
    pub fn get(
        &self
    ) -> *mut T {
        self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}
