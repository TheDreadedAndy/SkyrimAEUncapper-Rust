//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @author Kassent
//! @author Vadfromnu
//! @brief Top level library configuration and initialization.
//! @bug No known bugs.
//!

mod skyrim;
mod hook_wrappers;

use std::sync::atomic::{AtomicBool, Ordering};

use ctypes::cstr_array;
use skse64::log::{skse_message, skse_error};
use skse64::version::{SkseVersion, PACKED_SKSE_VERSION, CURRENT_RELEASE_RUNTIME};
use skse64::plugin_api::{SKSEPluginVersionData, SKSEInterface};
use skse64::plugin_api::SKSEPluginVersionData_kVersion;
use skse64::plugin_api::SKSEPluginVersionData_kVersionIndependentEx_NoStructUse;
use skse64::plugin_api::SKSEPluginVersionData_kVersionIndependent_AddressLibraryPostAE;
use skse64::utilities::get_runtime_dir;

/// @brief SKSE version structure (post-AE).
#[no_mangle]
pub static SKSEPlugin_Version: SKSEPluginVersionData = SKSEPluginVersionData {
    dataVersion: SKSEPluginVersionData_kVersion as u32,
    pluginVersion: 2,
    name: cstr_array("SkyrimUncapperAE"),
    author: cstr_array("Andrew Spaulding (Kasplat)"),
    supportEmail: cstr_array("andyespaulding@gmail.com"),
    versionIndependenceEx: SKSEPluginVersionData_kVersionIndependentEx_NoStructUse as u32,
    versionIndependence: SKSEPluginVersionData_kVersionIndependent_AddressLibraryPostAE as u32,
    compatibleVersions: [0; 16usize],
    seVersionRequired: 0
};

///
/// @brief SKSE plugin entrypoint.
///
/// Called by SKSE when our plugin is loaded.
///
/// The given interface must be valid for this to be safe.
///
#[no_mangle]
pub unsafe extern "system" fn SKSEPlugin_Load(
    skse: *const SKSEInterface
) -> bool {
    // "yup no more editor" ~ianpatt
    if (*skse).isEditor != 0 { return false; }

    // Prevent reinit.
    static IS_INIT: AtomicBool = AtomicBool::new(false);
    if IS_INIT.swap(true, Ordering::Relaxed) {
        skse_error!("Cannot reinitialize library!");
        return true;
    }

    // Create/open log file.
    let log_file = dirs_next::document_dir().unwrap().join(
        "My Games/Skyrim Special Edition/SKSE/SkyrimUncapper.log"
    );
    skse64::log::open(&log_file);

    // Log runtime/skse info.
    skse_message!(
        "Compiled: SKSE64 = {}, Skyrim AE = {}\nRunning: SKSE64 = {}, Skyrim AE = {}",
        PACKED_SKSE_VERSION,
        CURRENT_RELEASE_RUNTIME,
        SkseVersion::from_raw((*skse).skseVersion),
        SkseVersion::from_raw((*skse).runtimeVersion)
    );

    // TODO: Load settings.
    let _ini_path = get_runtime_dir().join("data/SKSE/SkyrimUncapper.ini");

    // TODO: Apply game patches.

    skse_message!("Initialization complete!");
    return true;
}
