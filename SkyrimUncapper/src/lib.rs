//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @author Kassent
//! @author Vadfromnu
//! @brief Top level library configuration and initialization.
//! @bug No known bugs.
//!

// Our crate name is stupid, for historical reasons.
#![allow(non_snake_case)]

mod skyrim;
mod hook_wrappers;
mod hooks;
mod settings;
mod patcher;
mod safe;

use ctypes::cstr_array;
use skse64::log::skse_message;
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
    name: cstr_array("SkyrimUncapper"),
    author: cstr_array("Andrew Spaulding (Kasplat)"),
    supportEmail: cstr_array("andyespaulding@gmail.com"),
    versionIndependenceEx: SKSEPluginVersionData_kVersionIndependentEx_NoStructUse as u32,
    versionIndependence: SKSEPluginVersionData_kVersionIndependent_AddressLibraryPostAE as u32,
    compatibleVersions: [0; 16usize],
    seVersionRequired: 0
};

///
/// Plugin entry point.
///
/// Called by the SKSE64 crate when our plugin is loaded. This function will only be called once.
///
#[no_mangle]
pub fn skse_plugin_rust_entry(
    skse: &SKSEInterface
) -> Result<(), ()> {
    // Log runtime/skse info.
    skse_message!(
        "Compiled: SKSE64 {}, Skyrim AE {}\nRunning: SKSE64 {}, Skyrim AE {}\nBase addr: {:#x}",
        PACKED_SKSE_VERSION,
        CURRENT_RELEASE_RUNTIME,
        SkseVersion::from_raw(skse.skseVersion),
        SkseVersion::from_raw(skse.runtimeVersion),
        skse64::reloc::RelocAddr::base()
    );

    settings::init(&get_runtime_dir().join("data\\SKSE\\Plugins\\SkyrimUncapper.ini"));
    patcher::apply()?;
    skse_message!("Initialization complete!");
    panic!("Haha! You were laboring under the misapprehension that we loaded successfully!");
    Ok(())
}
