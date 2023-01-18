//!
//! @file lib.rs
//! @author Andrew Spaulding (aspauldi)
//! @brief Exposes a Rust-safe interface to the address independence library.
//! @bug No known bugs.
//!

// We don't need it.
#![no_std]

use core::ffi::{c_void, c_int, c_ulonglong};
use core::ptr::NonNull;

extern "C" {
    fn VersionLibDb__create__() -> *mut c_void;
    fn VersionLibDb__destroy__(db: *mut c_void);
    fn VersionLibDb__find_offset_by_id__(
        db: *mut c_void,
        id: c_ulonglong,
        result: *mut c_ulonglong
    ) -> c_int;
    fn VersionLibDb__find_id_by_offset__(
        db: *mut c_void,
        offset: c_ulonglong,
        result: *mut c_ulonglong
    ) -> c_int;
}

/// @brief Wraps the pointer to the version lib db, automatically destroying it when dropped.
pub struct VersionDb(NonNull<c_void>);

impl VersionDb {
    ///
    /// @brief Attempts to create a new version database, and load it with the running version.
    ///
    pub fn new() -> Result<VersionDb, ()> {
        unsafe {
            // SAFETY: This ffi call is actually safe.
            let db = VersionLibDb__create__();
            NonNull::new(db).ok_or(()).map(|d| VersionDb(d))
        }
    }

    ///
    /// @brief Attempts to find the address independent id for the given offset.
    ///
    pub fn find_id_by_offset(
        &self,
        offset: usize
    ) -> Result<usize, ()> {
        let mut res: c_ulonglong = 0;
        let offset = offset as c_ulonglong;
        unsafe {
            // SAFETY: We know our result pointer and version Db are valid.
            if VersionLibDb__find_id_by_offset__(self.0.as_ptr(), offset, &mut res) >= 0 {
                Ok(res as usize)
            } else {
                Err(())
            }
        }
    }

    ///
    /// @brief Attempts to find the offset of the given address independent id.
    ///
    pub fn find_offset_by_id(
        &self,
        id: usize
    ) -> Result<usize, ()> {
        let mut res: c_ulonglong = 0;
        let id = id as c_ulonglong;
        unsafe {
            // SAFETY: We know our result pointer and version Db are valid.
            if VersionLibDb__find_offset_by_id__(self.0.as_ptr(), id, &mut res) >= 0 {
                Ok(res as usize)
            } else {
                Err(())
            }
        }
    }
}

impl Drop for VersionDb {
    fn drop(
        &mut self
    ) {
        unsafe {
            // SAFETY: We know our version DB pointer is valid.
            VersionLibDb__destroy__(self.0.as_ptr());
        }
    }
}
