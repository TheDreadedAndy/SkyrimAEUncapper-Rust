//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat).
//! @brief Top level module file for SKSE64 reimplementation.
//! @bug No known bugs.
//!

#![no_std]
extern crate alloc;

pub use sre_common::skse64::reloc;

mod errors;
pub mod event;
pub mod log;
pub mod loader;

// Needed for macros
pub use core;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Runtime extensions of SKSE modules
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Exports the SKSE version API, adding methods for getting the running game/skse version.
pub mod version {
    pub use sre_common::skse64::version::*;
    use core_util::Later;

    /// Holds the running game/skse version. Initialized by the entry point.
    pub (in crate) static RUNNING_GAME_VERSION: Later<SkseVersion> = Later::new();
    pub (in crate) static RUNNING_SKSE_VERSION: Later<SkseVersion> = Later::new();

    /// Gets the currently running game version.
    pub fn current_runtime() -> SkseVersion {
        *RUNNING_GAME_VERSION
    }

    /// Gets the currently running SKSE version.
    pub fn current_skse() -> SkseVersion {
        *RUNNING_SKSE_VERSION
    }
}

/// Exports the SKSE plugin API, adding a method for obtaining the current plugin handle.
pub mod plugin_api {
    use core::ffi::c_char;

    pub use sre_common::skse64::plugin_api::*;
    use core_util::Later;

    use crate::version::SkseVersion;

    /// Holds the plugin handle for this plugin.
    pub (in crate) static PLUGIN_HANDLE: Later<PluginHandle> = Later::new();

    /// Gets the handle for this plugin.
    pub fn handle() -> PluginHandle {
        *PLUGIN_HANDLE
    }

    /// A macro to construct SKSE version data.
    #[macro_export]
    macro_rules! plugin_version_data {
        (
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
                plugin_version: SkseVersion::new(
                    $crate::plugin_api::unsigned_from_str(
                        $crate::core::env!("CARGO_PKG_VERSION_MAJOR")),
                    $crate::plugin_api::unsigned_from_str(
                        $crate::core::env!("CARGO_PKG_VERSION_MINOR")),
                    $crate::plugin_api::unsigned_from_str(
                        $crate::core::env!("CARGO_PKG_VERSION_PATCH")),
                    $crate::plugin_api::unsigned_from_str(
                        $crate::core::env!("CARGO_PKG_VERSION_PRE"))
                ),
                name: $crate::plugin_api::make_str($crate::core::env!("CARGO_CRATE_NAME")),
                author: $crate::plugin_api::make_str($author),
                support_email: $crate::plugin_api::make_str($email),
                version_indep_ex: $vix,
                version_indep: $vi,
                compat_versions: $crate::plugin_api::make_vers(&[$($compat),*]),
                se_version_required: None
            };
        };
    }
    pub use plugin_version_data;

    // Converts strings to ints in const context, for version numbers.
    #[doc(hidden)]
    pub const fn unsigned_from_str(
        s: &str
    ) -> u32 {
        let s = s.as_bytes();
        let mut i = 0;
        let mut res = 0;
        while i < s.len() {
            assert!(b'0' <= s[i] && s[i] <= b'9');
            res *= 10;
            res += (s[i] - b'0') as u32;
            i += 1;
        }
        return res;
    }

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
