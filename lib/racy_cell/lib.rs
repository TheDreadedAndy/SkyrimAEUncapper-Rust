//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Thin wrapper around unsafe cell to mark it as sync. Allows one to avoid static mut.
//! @bug No known bugs.
//!
//! This code comes from the discussion on this thread:
//! https://github.com/rust-lang/rust/issues/53639#issuecomment-415515748
//!

#![no_std]

use core::cell::UnsafeCell;

#[repr(transparent)]
pub struct RacyCell<T>(UnsafeCell<T>);

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
