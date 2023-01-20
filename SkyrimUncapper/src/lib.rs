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

use ctypes::cstr_array;
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

    // TODO: Prevent reinit.

    // TODO: Open log file.

    // TODO: Print runtime/skse info.

    // TODO: Load settings.
    let _ini_path = get_runtime_dir().join("data/SKSE/SkyrimUncapper.ini");

    // TODO: Apply game patches.

    return true;
}
