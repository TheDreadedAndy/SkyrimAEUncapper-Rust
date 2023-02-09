//!
//! @file loader.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Loads in the skse plugin using the information provided by the linked crate.
//! @bug No known bugs.
//!

use std::sync::atomic::{AtomicBool, Ordering};

use crate::version::{RUNNING_GAME_VERSION, RUNNING_SKSE_VERSION, RUNTIME_VERSION_1_5_97};
use crate::plugin_api::{SkseInterface, SksePluginVersionData, PluginInfo};
use crate::reloc::RelocAddr;
use crate::errors::skse_panic;
use crate::log;

extern "Rust" {
    /// Entry point for plugins using this crate.
    fn skse_plugin_rust_entry(skse: &SkseInterface) -> Result<(), ()>;

    /// Used to name the log file.
    pub (in crate) static SKSEPlugin_Version: SksePluginVersionData;
}

///
/// SKSE plugin query function.
///
/// Only really necessary for SE, as part of the load process.
///
#[no_mangle]
pub unsafe extern "system" fn SKSEPlugin_Query(
    skse: *const SkseInterface,
    info: *mut PluginInfo
) -> bool {
    assert!(!skse.is_null());
    assert!(!info.is_null());
    *info = PluginInfo::from_ae(&SKSEPlugin_Version);

    // If this is ever false, then I will have several questions.
    return (*skse).runtime_version.unwrap() <= RUNTIME_VERSION_1_5_97;
}

///
/// SKSE plugin entry point.
///
/// Wraps the safe rust entry point for SKSE plugins, converting the interface to
/// something more "Rust" like and performing any necessary initialization.
///
#[no_mangle]
pub unsafe extern "system" fn SKSEPlugin_Load(
    skse: *const SkseInterface
) -> bool {
    // Prevent reinit.
    static IS_INIT: AtomicBool = AtomicBool::new(false);
    if IS_INIT.swap(true, Ordering::Relaxed) {
        log::skse_error!("Cannot reinitialize library!");
        return false;
    }

    RelocAddr::init_manager();

    // Set panics to print to the log (if it exists) and halt the plugin.
    std::panic::set_hook(Box::new(skse_panic));

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
