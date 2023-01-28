//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Simplified OnceCell that just panics on error.
//! @bug No known bugs.
//!

#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;

/// The core later structure. It is illegal to deref it before initialization.
pub struct Later<T> {
    is_init: AtomicBool,
    pl: UnsafeCell<MaybeUninit<T>>
}

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
        unsafe {
            // SAFETY: We have ensured that we are the only object initializing the data.
            (*self.pl.get()).write(pl);
        }
    }
}

impl<T> Deref for Later<T> {
    type Target = T;
    fn deref(
        &self
    ) -> &Self::Target {
        assert!(self.is_init.load(Ordering::Relaxed));
        unsafe {
            // SAFETY: We have ensured that the object is initialized.
            (*self.pl.get()).assume_init_ref()
        }
    }
}

impl<T> Drop for Later<T> {
    fn drop(
        &mut self
    ) {
        if *self.is_init.get_mut() {
            unsafe {
                // SAFETY: We have ensured that the object is initialized.
                (*self.pl.get()).assume_init_drop();
            }
        }
    }
}

// Later is sync if T is.
unsafe impl<T: Sync> Sync for Later<T> {}
