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

use core::fmt;
use core::fmt::{Arguments, Write};
use core::ffi::CStr;
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

/// Creates a CStr literal from a string literal.
#[macro_export]
macro_rules! cstr {
    ( $str:literal ) => {
        $crate::core::ffi::CStr::from_bytes_until_nul(
            $crate::core::concat!($str, "\0").as_bytes()
        ).unwrap()
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Anti-allocation goop
////////////////////////////////////////////////////////////////////////////////////////////////////

///
/// A structure for formatting strings into pre-allocated storage, which allows for text to be
/// calculated at runtime without causing a heap allocation.
///
/// The buffer is always ended with a null terminator, to allow for usage with FFI.
///
pub struct StringBuffer<const SIZE: usize> {
    buf: [u8; SIZE],
    len: usize
}

impl<const SIZE: usize> StringBuffer<SIZE> {
    /// Creates a new, empty, string buffer.
    pub const fn new() -> Self {
        Self {
            buf: [0; SIZE],
            len: 0
        }
    }

    /// Attempts to convert the contents of the buffer to a CStr.
    pub fn as_c_str(
        &self
    ) -> Result<&CStr, ()> {
        CStr::from_bytes_with_nul(self.as_bytes_nul()).map_err(|_| ())
    }

    /// Gets the underlying &[u8] in the buffer, with the null.
    pub fn as_bytes_nul(
        &self
    ) -> &[u8] {
        self.buf.split_at(self.len + 1).0
    }

    /// Formats the given arguments into the buffer, adding a newline.
    pub fn formatln(
        &mut self,
        args: Arguments<'_>
    ) -> Result<(), fmt::Error> {
        fmt::write(self, args)?;
        self.write_str("\n")?;
        Ok(())
    }

    ///
    /// Calls the given function, then updates the length of the buffer based on the null
    /// terminator.
    ///
    /// The given function must null terminate any data it appends.
    ///
    pub unsafe fn write_ffi(
        &mut self,
        func: impl FnOnce(&mut [u8])
    ) {
        func(self.buf.split_at_mut(self.len).1);

        while self.buf[self.len] != 0 {
            self.len += 1;
        }
    }

    /// Erases the contents of the buffer.
    pub fn clear(
        &mut self
    ) {
        self.buf[0] = 0;
        self.len = 0;
    }
}

impl<const SIZE: usize> fmt::Write for StringBuffer<SIZE> {
    fn write_str(
        &mut self,
        s: &str
    ) -> Result<(), fmt::Error> {
        if s.len() + self.len > SIZE - 1 {
            return Err(fmt::Error);
        }

        self.buf.split_at_mut(self.len).1.split_at_mut(s.len()).0.copy_from_slice(s.as_bytes());
        self.len += s.len();
        self.buf[self.len] = 0; // Always null terminate.
        Ok(())
    }
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
