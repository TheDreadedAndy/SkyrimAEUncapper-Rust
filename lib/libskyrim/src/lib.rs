//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat).
//! @brief Module runtime loader and environment for libskyrim.
//! @bug No known bugs.
//!

#![no_std]
extern crate alloc;

pub use sre_common::skse64::reloc;
pub mod patcher;
pub mod log;
pub mod ini;

// Needed for macros
pub use core;

use core::ptr;

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;

use core_util::RacyCell;

use crate::version::{RUNNING_GAME_VERSION, RUNNING_SKSE_VERSION, RUNTIME_VERSION_1_5_97};
use crate::plugin_api::{SkseInterface, SksePluginVersionData, PluginInfo, PLUGIN_HANDLE};
use crate::reloc::RelocAddr;
use crate::errors::SKSE_LOADER_DONE;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Core plugin loader
////////////////////////////////////////////////////////////////////////////////////////////////////
// Loads the linked application as a SKSE plugin, functioning with both AE and SE. It is expected
// that the linked application define a specific entry point symbol and a SKSEPlugin_Version
// descriptor symbol.

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

    // Initialize the relocation manager with the base address of skyrims binary.
    RelocAddr::init_manager(unsafe { GetModuleHandleA(ptr::null_mut()) as usize });

    // Set running version to the given value.
    RUNNING_SKSE_VERSION.init((*skse).skse_version.unwrap());
    RUNNING_GAME_VERSION.init((*skse).runtime_version.unwrap());

    // Open up a log file. I live in fear of errors before this point.
    log::open();

    // "yup, no more editor. obscript is gone (mostly)" ~ianpatt
    assert!(!skse.is_null());
    if (*skse).is_editor != 0 {
        *DO_ONCE.get() = Some(false);
        return false;
    }

    // Get our plugin handle and set up our SKSE listener.
    PLUGIN_HANDLE.init(((*skse).get_plugin_handle)());
    plugin_api::init_listener(skse.as_ref().unwrap());

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

    // Give SKSE our plugin info.
    *info = PluginInfo {
        info_version: PluginInfo::VERSION,
        name: SKSEPlugin_Version.name.as_ptr(),
        version: Some(SKSEPlugin_Version.plugin_version)
    };

    // If this is ever false, then I will have several questions.
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
        unsafe {
            // SAFETY: Protected by single threaded init.
            *SKSE_LOADER_DONE.get() = true;
        }
        return true;
    } else {
        // We must embrace pain and burn it as fuel for our journey.
        return false;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Runtime extensions of SKSE modules
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Exports the SKSE version API, adding methods for getting the running game/skse version.
pub mod version {
    pub use sre_common::skse64::version::*;
    use core_util::Later;

    /// Holds the running game/skse version. Initialized by the entry point.
    pub (in crate) static RUNNING_GAME_VERSION: Later<SkseVersion> = Later::new();
    pub (in crate) static RUNNING_SKSE_VERSION: Later<SkseVersion> = Later::new();

    /// Gets the currently running game version.
    pub fn current_runtime() -> SkseVersion {
        *RUNNING_GAME_VERSION
    }

    /// Gets the currently running SKSE version.
    pub fn current_skse() -> SkseVersion {
        *RUNNING_SKSE_VERSION
    }
}

/// Exports the SKSE plugin API, adding a method for obtaining the current plugin handle.
///
/// Additionally, defines a Rust-safe method for registering event listeners.
pub mod plugin_api {
    use core::ffi::c_char;
    use alloc::vec::Vec;

    pub use sre_common::skse64::plugin_api::*;
    use core_util::{Later, RacyCell};

    use crate::plugin_api;
    use crate::version::SkseVersion;

    // Vector initializer for skse handler array. Basically a language quirk.
    const VEC_INIT: Vec<fn(&Message)> = Vec::new();

    /// Holds the list of registered SKSE listeners to be called when SKSE invokes the event
    /// handler.
    static SKSE_HANDLERS: RacyCell<[Vec<fn(&Message)>; Message::SKSE_MAX]>
                                                    = RacyCell::new([VEC_INIT; Message::SKSE_MAX]);

    /// Holds the plugin handle for this plugin.
    pub (in crate) static PLUGIN_HANDLE: Later<PluginHandle> = Later::new();

    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// Registers our listener wrapper to the SKSE message sender.
    pub (in crate) fn init_listener(
        skse: &SkseInterface
    ) {
        unsafe {
            // SAFETY: The SkseInterface structure is provided by SKSE and is valid.
            let msg_if = (skse.query_interface)(InterfaceId::Messaging)
                         as *mut SkseMessagingInterface;
            ((*msg_if).register_listener)(
                plugin_api::handle(),
                "SKSE\0".as_bytes().as_ptr() as *const c_char,
                skse_listener
            );
        }
    }

    /// Registers a new listener for a skse message.
    pub fn register_listener(
        msg_type: u32,
        callback: fn(&Message)
    ) {
        assert!(msg_type < Message::SKSE_MAX as u32);
        unsafe {
            (*SKSE_HANDLERS.get())[msg_type as usize].push(callback);
        }
    }

    /// Gets the handle for this plugin.
    pub fn handle() -> PluginHandle {
        *PLUGIN_HANDLE
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// A macro to construct SKSE version data.
    #[macro_export]
    macro_rules! plugin_version_data {
        (
            author: $author:literal,
            email: $email:literal,
            version_indep_ex: $vix:expr,
            version_indep: $vi:expr,
            compat_versions: [ $($compat:expr),* ]
        ) => {
            #[no_mangle]
            pub static SKSEPlugin_Version: $crate::plugin_api::SksePluginVersionData =
            $crate::plugin_api::SksePluginVersionData {
                data_version: $crate::plugin_api::SksePluginVersionData::VERSION,
                plugin_version: SkseVersion::new(
                    $crate::plugin_api::unsigned_from_str(
                        $crate::core::env!("CARGO_PKG_VERSION_MAJOR")),
                    $crate::plugin_api::unsigned_from_str(
                        $crate::core::env!("CARGO_PKG_VERSION_MINOR")),
                    $crate::plugin_api::unsigned_from_str(
                        $crate::core::env!("CARGO_PKG_VERSION_PATCH")),
                    $crate::plugin_api::unsigned_from_str(
                        $crate::core::env!("CARGO_PKG_VERSION_PRE"))
                ),
                name: $crate::plugin_api::make_str($crate::core::env!("CARGO_CRATE_NAME")),
                author: $crate::plugin_api::make_str($author),
                support_email: $crate::plugin_api::make_str($email),
                version_indep_ex: $vix,
                version_indep: $vi,
                compat_versions: $crate::plugin_api::make_vers(&[$($compat),*]),
                se_version_required: None
            };
        };
    }
    pub use plugin_version_data;

    // Converts strings to ints in const context, for version numbers.
    #[doc(hidden)]
    pub const fn unsigned_from_str(
        s: &str
    ) -> u32 {
        let s = s.as_bytes();
        let mut i = 0;
        let mut res = 0;
        while i < s.len() {
            assert!(b'0' <= s[i] && s[i] <= b'9');
            res *= 10;
            res += (s[i] - b'0') as u32;
            i += 1;
        }
        return res;
    }

    // Converts an ascii string literal to a C string array.
    #[doc(hidden)]
    pub const fn make_str<const N: usize>(
        s: &str
    ) -> [c_char; N] {
        let mut ret: [c_char; N] = [0; N];

        let s = s.as_bytes();
        assert!(s.len() <= (N - 1), "Cannot fit string in C array!");

        let mut i = 0;
        while i < s.len() {
            ret[i] = s[i] as i8;
            i += 1;
        }

        ret
    }

    // Converts a list of SKSE versions into a compatible versions array.
    #[doc(hidden)]
    pub const fn make_vers<const N: usize>(
        v: &[SkseVersion]
    ) -> [Option<SkseVersion>; N] {
        let mut ret = [None; N];
        assert!(v.len() <= (N - 1), "Too many compatible versions!");

        let mut i = 0;
        while i < v.len() {
            ret[i] = Some(v[i]);
            i += 1;
        }

        ret
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// Handles a message from the skse plugin by forwarding it to the registered listener.
    unsafe extern "system" fn skse_listener(
        msg: *mut Message
    ) {
        let msg = msg.as_ref().unwrap();

        // Only handle messages we understand.
        if msg.msg_type >= Message::SKSE_MAX as u32 { return; }

        for callback in (*SKSE_HANDLERS.get())[msg.msg_type as usize].iter() {
            callback(msg);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Skyrim runtime environment panic implementation
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Provides an implementation of panic which shows a pop-up message and logs an error when called.
// The implementation will also abort using a system exception during the plugin loading phase,
// which the AE version of SKSE can catch.
//
// Note that there is a bug in the AE version of SKSE which means that we cannot always abort using
// a system exception, as doing so in later phases will cause SKSE to misbehave. Thus, once the
// loader has finished we instead terminate using the C standard abort() function.

// Private, since it only provides a panic implementation.
mod errors {
    use core_util::RacyCell;
    use crate::log;

    extern "system" {
        /// Halts the loading of a SKSE plugin.
        fn stop_plugin() -> !;
    }

    // C standard abort, for post-load panics.
    #[link(name = "msvcrt")]
    extern "C" { fn abort() -> !; }

    // Implement stop_plugin().
    core::arch::global_asm! {
        include_str!("stop_plugin.S"),
        options(att_syntax)
    }

    pub (in crate) static SKSE_LOADER_DONE: RacyCell<bool> = RacyCell::new(false);

    /// Stops the loading of the plugin when called during the load phase.
    #[panic_handler]
    fn skse_panic(
        info: &core::panic::PanicInfo<'_>
    ) -> ! {
        log::skse_fatal!("{}", info);
        unsafe {
            // After loading has finished, it's not safe to halt the plugin by throwing an
            // exception. This is due to a bug in SKSE where exceptions are caught and then ignored
            // if they are thrown during the messaging phases. As such, we abort on panic if it
            // happens after the loading phase has finished.
            if *SKSE_LOADER_DONE.get() {
                abort();
            } else {
                stop_plugin();
            }
        }
    }
}
