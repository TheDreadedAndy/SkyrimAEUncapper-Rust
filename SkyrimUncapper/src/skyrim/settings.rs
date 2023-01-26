//!
//! @file settings.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines the settings structure used by Skyrim.
//! @bug No known bugs.
//!

use std::ffi::c_char;
use std::cell::UnsafeCell;

use ctypes::abstract_type;
use skse64::errors::skse_assert;

abstract_type! {
    /// Contains configuration settings exposed by the game engine.
    pub type SettingCollectionMap;
}

/// The union of valid settings data types within the game.
pub union SettingData {
    u: u32,
    i: i32,
    f: f32,
    b: u8,
    s: *mut c_char
}

/// The settings structure, as defined by skyrim.
pub struct Setting {
    vtbl: *const (),
    data: UnsafeCell<SettingData>,
    name: *mut c_char
}

impl Setting {
    /// Gets the underlying floating point value of the setting.
    pub fn get_float(
        &self
    ) -> f32 {
        unsafe {
            // SAFETY: We ensure that the underlying type is a float.
            skse_assert!(*self.name == b'f' as i8);
            (*self.data.get()).f
        }
    }

    /// Sets the underlying floating point value of the setting.
    pub fn set_float(
        &self,
        f: f32
    ) {
        unsafe {
            // SAFETY: We ensure that the underlying type is a float.
            skse_assert!(*self.name == b'f' as i8);
            (*self.data.get()).f = f;
        }
    }
}
