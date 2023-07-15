//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat).
//! @brief Top level module file for SKSE64 reimplementation.
//! @bug No known bugs.
//!

pub use sre_common::skse64::reloc;

pub mod version;
pub mod event;
mod errors;
pub mod log;
pub mod loader;

/// Exports the SKSE plugin API, adding a method for obtaining the current plugin handle.
pub mod plugin_api {
    use core_util::Later;

    pub use sre_common::skse64::plugin_api::*;

    /// Holds the plugin handle for this plugin.
    pub (in crate) static PLUGIN_HANDLE: Later<PluginHandle> = Later::new();

    /// Gets the handle for this plugin.
    pub fn handle() -> PluginHandle {
        *PLUGIN_HANDLE
    }
}
