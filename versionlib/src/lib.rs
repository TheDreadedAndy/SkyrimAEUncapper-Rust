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

use skse64::version::SkseVersion;

extern "C" {
    fn VersionLibDb__init__() -> NonNull<c_void>;
    fn VersionLibDb__destroy__(db: NonNull<c_void>);
    fn VersionLibDb__load_current__(db: NonNull<c_void>);
    fn VersionLibDb__load_release__(
        db: NonNull<c_void>,
        major: c_int,
        minor: c_int,
        build: c_int,
        sub: c_int
    );
    fn VersionLibDb__find_offset_by_id__(
        db: NonNull<c_void>,
        id: c_ulonglong,
        result: *mut c_ulonglong
    ) -> c_int;
    fn VersionLibDb__find_id_by_offset__(
        db: NonNull<c_void>,
        offset: c_ulonglong,
        result: *mut c_ulonglong
    ) -> c_int;
}

/// @brief The VersionDb wrapper struct. Applies constructors and destructors.
pub struct VersionDb(NonNull<c_void>);

impl VersionDb {
    ///
    /// @brief Attempts to create a new version database, and loads it with the specified version
    ///        (or the current version, if none is provided).
    ///
    pub fn new(
        version: Option<SkseVersion>
    ) -> Self {
        unsafe {
            // SAFETY: This ffi call is actually safe.
            let db = VersionDb(VersionLibDb__init__());

            // Load the database.
            if let Some(v) = version {
                VersionLibDb__load_release__(
                    db.0,
                    v.major() as c_int,
                    v.minor() as c_int,
                    v.build() as c_int,
                    v.runtime_type() as c_int
                );
            } else {
                VersionLibDb__load_current__(db.0);
            }

            return db;
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
            if VersionLibDb__find_id_by_offset__(self.0, offset, &mut res) >= 0 {
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
            if VersionLibDb__find_offset_by_id__(self.0, id, &mut res) >= 0 {
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
            VersionLibDb__destroy__(self.0);
        }
    }
}
