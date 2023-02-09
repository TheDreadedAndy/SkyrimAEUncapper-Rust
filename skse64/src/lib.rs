//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat).
//! @brief Top level module file for SKSE64 reimplementation.
//! @bug No known bugs.
//!

pub mod version;
pub mod util;

#[cfg(not(feature = "not_plugin"))] mod errors;
#[cfg(not(feature = "not_plugin"))] pub mod log;
#[cfg(not(feature = "not_plugin"))] pub mod reloc;
#[cfg(not(feature = "not_plugin"))] pub mod plugin_api;
#[cfg(all(feature = "trampoline", not(feature = "not_plugin")))] pub mod trampoline;
#[cfg(not(feature = "not_plugin"))] pub mod safe;

// For macros.
pub use core;

#[cfg(not(feature = "not_plugin"))] use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(feature = "not_plugin"))] use version::{RUNNING_GAME_VERSION, RUNNING_SKSE_VERSION};
#[cfg(not(feature = "not_plugin"))] use plugin_api::{SkseInterface, SksePluginVersionData};

#[cfg(not(feature = "not_plugin"))]
extern "Rust" {
    /// Entry point for plugins using this crate.
    fn skse_plugin_rust_entry(skse: &SkseInterface) -> Result<(), ()>;

    /// Used to name the log file.
    pub (in crate) static SKSEPlugin_Version: SksePluginVersionData;
}

///
/// SKSE plugin entry point.
///
/// Wraps the safe rust entry point for SKSE plugins, converting the interface to
/// something more "Rust" like and performing any necessary initialization.
///
#[no_mangle]
#[cfg(not(feature = "not_plugin"))]
pub unsafe extern "system" fn SKSEPlugin_Load(
    skse: *const SkseInterface
) -> bool {
    // Prevent reinit.
    static IS_INIT: AtomicBool = AtomicBool::new(false);
    if IS_INIT.swap(true, Ordering::Relaxed) {
        skse_error!("Cannot reinitialize library!");
        return false;
    }

    reloc::RelocAddr::init_manager();

    // Set panics to print to the log (if it exists) and halt the plugin.
    std::panic::set_hook(Box::new(errors::skse_panic));

    // Before we do anything else, we try and open up a log file.
    log::open();

    // "yup, no more editor. obscript is gone (mostly)" ~ianpatt
    assert!(!(skse.is_null()));
    if (*skse).is_editor != 0 { return false; }

    // Set running version to the given value.
    RUNNING_SKSE_VERSION.init((*skse).skse_version.unwrap());
    RUNNING_GAME_VERSION.init((*skse).runtime_version.unwrap());

    // Call the rust entry point.
    return skse_plugin_rust_entry(skse.as_ref().unwrap()).is_ok();
}
