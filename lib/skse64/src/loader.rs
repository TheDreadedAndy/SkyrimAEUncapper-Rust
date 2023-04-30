//!
//! @file loader.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Loads in the skse plugin using the information provided by the linked crate.
//! @bug No known bugs.
//!

use racy_cell::RacyCell;

use crate::version::{RUNNING_GAME_VERSION, RUNNING_SKSE_VERSION, RUNTIME_VERSION_1_5_97};
use crate::plugin_api::{SkseInterface, SksePluginVersionData, PluginInfo, PLUGIN_HANDLE};
use crate::event::init_listener;
use crate::reloc::RelocAddr;
use crate::errors::{skse_loader_panic, skse_runtime_panic};
use crate::log;

extern "Rust" {
    /// Entry point for plugins using this crate.
    fn skse_plugin_rust_entry(skse: &SkseInterface) -> Result<(), ()>;

    /// Used to name the log file.
    pub (in crate) static SKSEPlugin_Version: SksePluginVersionData;
}

///
/// Initializes SKSE logging and addressing.
///
/// If this function is called more than once, only the first call will be acted on.
///
unsafe fn init_skse(
    skse: *const SkseInterface
) -> bool {
    static DO_ONCE: RacyCell<Option<bool>> = RacyCell::new(None);
    if let Some(ret) = *DO_ONCE.get() {
        return ret;
    }

    RelocAddr::init_manager();

    // Set panics to print to the log (if it exists) and halt the plugin.
    std::panic::set_hook(Box::new(skse_loader_panic));

    // Before we do anything else, we try and open up a log file.
    log::open();

    // "yup, no more editor. obscript is gone (mostly)" ~ianpatt
    assert!(!skse.is_null());
    if (*skse).is_editor != 0 {
        *DO_ONCE.get() = Some(false);
        return false;
    }

    // Set running version to the given value.
    RUNNING_SKSE_VERSION.init((*skse).skse_version.unwrap());
    RUNNING_GAME_VERSION.init((*skse).runtime_version.unwrap());

    // Get our plugin handle and set up our SKSE listener.
    PLUGIN_HANDLE.init(((*skse).get_plugin_handle)());
    init_listener(skse.as_ref().unwrap());

    *DO_ONCE.get() = Some(true);
    return true;
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
    if !init_skse(skse) { return false; }
    assert!(!info.is_null());

    // If this is ever false, then I will have several questions.
    *info = PluginInfo::from_ae(&SKSEPlugin_Version);
    if (*skse).runtime_version.unwrap() <= RUNTIME_VERSION_1_5_97 {
        log::skse_message!("Plugin query complete, marking as compatible.");
        return true;
    } else {
        log::skse_message!("Unknown game version. Marking as incompatible.");
        return false;
    }
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
    static DO_ONCE: RacyCell<bool> = RacyCell::new(true);
    if !*DO_ONCE.get() {
        log::skse_message!("Cannot reinitialize library!");
        return false;
    } else {
        *DO_ONCE.get() = false;
    }

    // If we're running on an AE version, we haven't done this yet.
    if !init_skse(skse) { return false; }

    // Call the rust entry point.
    if let Ok(_) = skse_plugin_rust_entry(skse.as_ref().unwrap()) {
        // All future panics must terminate skyrim.
        std::panic::set_hook(Box::new(skse_runtime_panic));
        return true;
    } else {
        // We must embrace pain and burn it as fuel for our journey.
        return false;
    }
}
