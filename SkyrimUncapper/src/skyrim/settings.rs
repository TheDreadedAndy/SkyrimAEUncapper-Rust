//!
//! @file settings.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines the settings structure used by Skyrim.
//! @bug No known bugs.
//!

use std::ffi::c_char;
use std::cell::UnsafeCell;

skse64::util::abstract_type! {
    /// Contains configuration settings exposed by the game engine.
    pub type SettingCollectionMap;
}

/// The union of valid settings data types within the game.
#[repr(C)]
pub union SettingData {
    _u: u32,
    _i: i32,
    f: f32,
    _b: u8,
    _s: *mut c_char
}

/// The settings structure, as defined by skyrim.
#[repr(C)]
pub struct Setting {
    _vtbl: *const (),
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
            assert!(*self.name == b'f' as i8);
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
            assert!(*self.name == b'f' as i8);
            (*self.data.get()).f = f;
        }
    }
}
