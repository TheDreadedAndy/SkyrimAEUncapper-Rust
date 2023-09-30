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

#![no_std]
extern crate alloc;

mod skyrim;
mod hooks;
mod settings;

// For macros.
pub use core;

use alloc::alloc::{GlobalAlloc, Layout};

use libskyrim::log::{skse_message, skse_fatal};
use libskyrim::plugin_api::{SksePluginVersionData, SkseInterface};
use libskyrim::patcher::flatten_patch_groups;

use skyrim::{GAME_SIGNATURES, NUM_GAME_SIGNATURES};
use hooks::{HOOK_SIGNATURES, NUM_HOOK_SIGNATURES};

////////////////////////////////////////////////////////////////////////////////////////////////////

// Since we're in a no_std environment, we need to define a memory allocator for the alloc crate to
// use.
struct SystemAlloc;

// These are defined in CRT, but not in libc.
extern "C" {
    fn _aligned_malloc(size: usize, align: usize) -> *mut u8;
    fn _aligned_free(ptr: *mut u8);
}

unsafe impl GlobalAlloc for SystemAlloc {
    unsafe fn alloc(
        &self,
        layout: Layout
    ) -> *mut u8 {
        _aligned_malloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(
        &self,
        ptr: *mut u8,
        _layout: Layout
    ) {
        _aligned_free(ptr);
    }
}

#[global_allocator]
static A: SystemAlloc = SystemAlloc;

////////////////////////////////////////////////////////////////////////////////////////////////////

const NUM_PATCHES: usize = NUM_GAME_SIGNATURES + NUM_HOOK_SIGNATURES;

libskyrim::plugin_api::plugin_version_data! {
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
    _skse: &SkseInterface
) -> Result<(), ()> {
    settings::init(core_util::cstr!("Data\\SKSE\\Plugins\\SkyrimUncapper.ini"));

    let patches = flatten_patch_groups::<NUM_PATCHES>(&[&GAME_SIGNATURES, &HOOK_SIGNATURES]);
    if let Err(_) = libskyrim::patcher::apply(patches) {
        skse_fatal!(
            "Failed to install the requested set of game patches. See log for details.\n\
             It is safe to continue playing; none of this mods changes have been applied."
        );
        return Err(());
    }

    skse_message!("Initialization complete!");
    Ok(())
}
