//!
//! @file plugin_api.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes the plugin API data structures.
//! @bug No known bugs.
//!

use core::ffi::{c_char, c_void};

use crate::version::SkseVersion;

/// A macro to construct SKSE version data.
#[macro_export]
macro_rules! plugin_version_data {
    (
        version: $ver:expr,
        name: $name:literal,
        author: $author:literal,
        email: $email:literal,
        version_indep_ex: $vix:expr,
        version_indep: $vi:expr,
        compat_versions: [ $($compat:expr),* ]
    ) => {
        #[no_mangle]
        pub static SKSEPlugin_Version: $crate::plugin_api::SksePluginVersionData =
        $crate::plugin_api::SksePluginVersionData {
            data_version: $crate::plugin_api::SksePluginVersionData::VERSION,
            plugin_version: $ver,
            name: $crate::plugin_api::SksePluginVersionData::make_str($name),
            author: $crate::plugin_api::SksePluginVersionData::make_str($author),
            support_email: $crate::plugin_api::SksePluginVersionData::make_str($email),
            version_indep_ex: $vix,
            version_indep: $vi,
            compat_versions: $crate::plugin_api::SksePluginVersionData::make_vers(&[$($compat),*]),
            se_version_required: None
        };
    };
}
pub use plugin_version_data;

#[repr(C)]
pub struct PluginInfo {
    pub info_version: u32,
    pub name: *const c_char,
    pub version: u32
}

#[repr(C)]
pub struct SkseInterface {
    pub skse_version: Option<SkseVersion>,
    pub runtime_version: Option<SkseVersion>,
    pub editor_version: u32,
    pub is_editor: u32,
    pub query_interface: extern "system" fn(u32) -> *mut c_void,
    pub get_plugin_handle: extern "system" fn() -> u32,
    pub get_release_index: extern "system" fn() -> u32,
    pub get_plugin_info: extern "system" fn(*const c_char) -> *const PluginInfo
}

#[repr(C)]
pub struct SksePluginVersionData {
    pub data_version: u32, // Self::VERSION
    pub plugin_version: SkseVersion,
    pub name: [c_char; 256], // Plugin name (can be empty).
    pub author: [c_char; 256], // Author name (can be empty).
    pub support_email: [c_char; 252], // Not shown to users. For SKSE team to contact mod maker.
    pub version_indep_ex: u32,
    pub version_indep: u32,
    pub compat_versions: [Option<SkseVersion>; 16], // None-terminated.
    pub se_version_required: Option<SkseVersion> // Minimum SKSE version required.
}

impl SksePluginVersionData {
    pub const VERSION: u32 = 1;

    // Set if plugin uses the address independence library.
    pub const VINDEP_ADDRESS_LIBRARY_POST_AE: u32 = 1 << 0;

    // Set if the plugin uses only signature scanning.
    pub const VINDEP_SIGNATURES: u32 = 1 << 1;

    // Set if the plugin uses 629+ compatible structs. 629+ won't load without this.
    pub const VINDEP_STRUCTS_POST_629: u32 = 1 << 2;

    // Allows the plugin to load with all AE versions. Only set if you don't use structs
    // or check your version before accessing them manually.
    pub const VINDEPEX_NO_STRUCT_USE: u32 = 1 << 0;

    // Converts an ascii string literal to a C string array.
    #[doc(hidden)]
    pub const fn make_str<const N: usize>(
        s: &str
    ) -> [c_char; N] {
        let mut ret: [c_char; N] = [0; N];

        let s = s.as_bytes();
        assert!(s.len() <= (N - 1), "Cannot fit string in C array!");

        let mut i = 0;
        while i < s.len() {
            ret[i] = s[i] as i8;
            i += 1;
        }

        ret
    }

    // Converts a list of SKSE versions into a compatible versions array.
    #[doc(hidden)]
    pub const fn make_vers<const N: usize>(
        v: &[SkseVersion]
    ) -> [Option<SkseVersion>; N] {
        let mut ret = [None; N];
        assert!(v.len() <= (N - 1), "Too many compatible versions!");

        let mut i = 0;
        while i < v.len() {
            ret[i] = Some(v[i]);
            i += 1;
        }

        ret
    }
}
