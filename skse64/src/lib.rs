//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat).
//! @brief Top level module file for SKSE FFI.
//! @bug No known bugs.
//!

mod bind;
pub mod version;
mod errors;
pub mod log;
pub mod reloc;
pub mod plugin_api;
pub mod trampoline;
pub mod safe;

// For macros.
pub use core;

use std::sync::atomic::{AtomicBool, Ordering};
use std::ffi::CStr;

use version::{RUNNING_GAME_VERSION, RUNNING_SKSE_VERSION};
use version::SkseVersion;
use plugin_api::{SKSEInterface, SKSEPluginVersionData};

extern "Rust" {
    /// Entry point for plugins using this crate.
    fn skse_plugin_rust_entry(skse: &SKSEInterface) -> Result<(), ()>;

    /// Used to name the log file.
    static SKSEPlugin_Version: SKSEPluginVersionData;
}

///
/// SKSE plugin entry point.
///
/// Wraps the safe rust entry point for SKSE plugins, converting the interface to
/// something more "Rust" like and performing any necessary initialization.
///
#[no_mangle]
pub unsafe extern "system" fn SKSEPlugin_Load(
    skse: *const SKSEInterface
) -> bool {
    // Prevent reinit.
    static IS_INIT: AtomicBool = AtomicBool::new(false);
    if IS_INIT.swap(true, Ordering::Relaxed) {
        skse_error!("Cannot reinitialize library!");
        return false;
    }

    // Set panics to print to the log (if it exists) and halt the plugin.
    std::panic::set_hook(Box::new(errors::skse_panic));

    // Before we do anything else, we try and open up a log file.
    log::open(format!(
        "{}.log",
        CStr::from_ptr(SKSEPlugin_Version.name.as_ptr()).to_str().unwrap()
    ));

    // "yup, no more editor. obscript is gone (mostly)" ~ianpatt
    assert!(!(skse.is_null()));
    if (*skse).isEditor != 0 { return false; }

    // Set running version to the given value.
    RUNNING_SKSE_VERSION.init(SkseVersion::from_raw((*skse).skseVersion));
    RUNNING_GAME_VERSION.init(SkseVersion::from_raw((*skse).runtimeVersion));

    // Call the rust entry point.
    return skse_plugin_rust_entry(skse.as_ref().unwrap()).is_ok();
}
