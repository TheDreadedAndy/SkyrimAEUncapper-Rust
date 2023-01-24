//!
//! @file settings.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines the settings structure used by Skyrim.
//! @bug No known bugs.
//!

use std::ffi::c_char;
use std::cell::UnsafeCell;

use skse64::errors::skse_assert;

/// The union of valid settings data types within the game.
pub union SettingsData {
    u: u32,
    i: i32,
    f: f32,
    b: u8,
    s: *mut c_char
}

/// The settings structure, as defined by skyrim.
pub struct Settings {
    vtbl: *const (),
    data: UnsafeCell<SettingsData>,
    name: *mut c_char
}

impl Settings {
    /// Gets the underlying floating point value of the setting.
    pub fn get_float(
        &self
    ) -> f32 {
        unsafe {
            // SAFETY: We ensure that the underlying type is a float.
            skse_assert!(*self.name == b'f');
            (*self.data.get()).f
        }
    }

    /// Sets the underlying floating point value of the setting.
    pub fn set_float(
        &self
        f: f32
    ) {
        unsafe {
            // SAFETY: We ensure that the underlying type is a float.
            skse_assert!(*self.name == b'f');
            (*self.data.get()).f = f;
        }
    }
}
