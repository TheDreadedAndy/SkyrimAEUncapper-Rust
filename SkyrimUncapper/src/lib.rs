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

use std::ffi::CStr;
use std::path::Path;

use skse64::log::{skse_message, skse_fatal};
use skse64::version::{SkseVersion, PACKED_SKSE_VERSION, CURRENT_RELEASE_RUNTIME};
use skse64::plugin_api::{SksePluginVersionData, SkseInterface};
use skyrim_patcher::flatten_patch_groups;

use skyrim::{GAME_SIGNATURES, NUM_GAME_SIGNATURES};
use hooks::{HOOK_SIGNATURES, NUM_HOOK_SIGNATURES};

// Get git version of the plugin.
include!(concat!(env!("OUT_DIR"), "/git_version.rs"));

const NUM_PATCHES: usize = NUM_GAME_SIGNATURES + NUM_HOOK_SIGNATURES;

skse64::plugin_version_data! {
    version: SkseVersion::new(2, 1, 0, 0),
    name: "SkyrimUncapper",
    author: "Andrew Spaulding (Kasplat)",
    email: "andyespaulding@gmail.com",
    version_indep_ex: SksePluginVersionData::VINDEPEX_NO_STRUCT_USE,
    version_indep: SksePluginVersionData::VINDEP_ADDRESS_LIBRARY_POST_AE,
    compat_versions: []
}

///
/// Plugin entry point.
///
/// Called by the SKSE64 crate when our plugin is loaded. This function will only be called once.
///
#[no_mangle]
pub fn skse_plugin_rust_entry(
    skse: &SkseInterface
) -> Result<(), ()> {
    // Log runtime/skse info.
    skse_message!(
        "{} {:?} ({})\n\
         Compiled: SKSE64 {}, Skyrim AE {}\n\
         Running: SKSE64 {}, Skyrim AE {}\n\
         Base addr: {:#x}",
        unsafe { CStr::from_ptr(SKSEPlugin_Version.name.as_ptr()).to_str().unwrap() },
        SKSEPlugin_Version.plugin_version,
        GIT_VERSION,
        PACKED_SKSE_VERSION,
        CURRENT_RELEASE_RUNTIME,
        (*skse).skse_version.unwrap(),
        (*skse).runtime_version.unwrap(),
        skse64::reloc::RelocAddr::base()
    );

    settings::init(Path::new("Data\\SKSE\\Plugins\\SkyrimUncapper.ini"));

    let patches = flatten_patch_groups::<NUM_PATCHES>(&[&GAME_SIGNATURES, &HOOK_SIGNATURES]);
    if let Err(_) = skyrim_patcher::apply(patches) {
        skse_fatal!(
            "Failed to install the requested set of game patches. See log for details.\n\
             It is safe to continue playing; none of this mods changes have been applied."
        );
        return Err(());
    }

    skse_message!("Initialization complete!");
    Ok(())
}
