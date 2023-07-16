//!
//! @file plugin_api.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes the plugin API data structures.
//! @bug No known bugs.
//!

use core::ffi::{c_char, c_void};

use crate::skse64::version::SkseVersion;

/// Plugin interface IDs.
#[repr(u32)]
pub enum InterfaceId {
    Invalid,
    Scaleform,
    Papyrus,
    Serialization,
    Task,
    Messaging,
    Object,
    Trampoline,
    Max
}

/// The ID assigned to a loaded plugin. SKSE docs request this be used as an abstract type.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PluginHandle(u32);

/// Plugin query info returned to skse for SE.
#[repr(C)]
pub struct PluginInfo {
    pub info_version: u32,
    pub name: *const c_char,
    pub version: Option<SkseVersion>
}

/// IMPORTANT: the bottom three fields DO NOT EXIST for SE.
#[repr(C)]
pub struct SkseInterface {
    pub skse_version: Option<SkseVersion>,
    pub runtime_version: Option<SkseVersion>,
    pub editor_version: u32,
    pub is_editor: u32,
    pub query_interface: unsafe extern "system" fn(InterfaceId) -> *mut c_void,
    pub get_plugin_handle: unsafe extern "system" fn() -> PluginHandle,
    pub get_release_index: unsafe extern "system" fn() -> u32,
    pub get_plugin_info: unsafe extern "system" fn(*const c_char) -> *const PluginInfo
}

/// A message which can be received from/sent to other skse plugins.
#[repr(C)]
pub struct Message {
    pub sender: *const c_char,
    pub msg_type: u32,
    pub data_len: u32,
    pub data: *mut u8
}

/// A callback function registered as a message listener.
pub type MessageCallback = unsafe extern "system" fn(*mut Message);

/// The interface SKSE returns for messaging it and other SKSE plugins.
#[repr(C)]
pub struct SkseMessagingInterface {
    pub interface_version: u32,
    pub register_listener: unsafe extern "system" fn(
        PluginHandle,
        *const c_char,
        MessageCallback
    ) -> bool,
    pub dispatch: unsafe extern "system" fn(
        PluginHandle,
        u32,
        *mut c_void,
        u32,
        *const c_char
    ) -> bool,
    pub get_event_dispatcher: unsafe extern "system" fn(u32) -> *mut c_void
}

/// Plugin info exported to skse for AE.
#[repr(C)]
pub struct SksePluginVersionData {
    pub data_version: u32, // Self::VERSION
    pub plugin_version: SkseVersion,
    pub name: [c_char; 256], // Plugin name (can be empty).
    pub author: [c_char; 256], // Author name (can be empty).
    pub support_email: [c_char; 252], // Not shown to users. For SKSE team to contact mod maker.
    pub version_indep_ex: u32,
    pub version_indep: u32,
    pub compat_versions: [Option<SkseVersion>; 16], // None-terminated.
    pub se_version_required: Option<SkseVersion> // Minimum SKSE version required.
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl PluginInfo {
    pub const VERSION: u32 = 1;

    /// Creates SE plugin info from an AE plugin data structure.
    pub fn from_ae(
        ae: &SksePluginVersionData
    ) -> Self {
        Self {
            info_version: Self::VERSION,
            name: ae.name.as_ptr(),
            version: Some(ae.plugin_version)
        }
    }
}

impl Message {
    // Messages which SKSE itself can send.
    pub const SKSE_POST_LOAD: u32 = 0;
    pub const SKSE_POST_POST_LOAD: u32 = 1;
    pub const SKSE_PRE_LOAD_GAME: u32 = 2;
    pub const SKSE_POST_LOAD_GAME: u32 = 3;
    pub const SKSE_SAVE_GAME: u32 = 4;
    pub const SKSE_DELETE_GAME: u32 = 5;
    pub const SKSE_INPUT_LOADED: u32 = 6;
    pub const SKSE_NEW_GAME: u32 = 7;
    pub const SKSE_DATA_LOADED: u32 = 8;
    pub const SKSE_MAX: usize = 9;
}

impl SkseMessagingInterface {
    pub const VERSION: u32 = 2;
}

impl SksePluginVersionData {
    pub const VERSION: u32 = 1;

    // Set if plugin uses the address independence library.
    pub const VINDEP_ADDRESS_LIBRARY_POST_AE: u32 = 1 << 0;

    // Set if the plugin uses only signature scanning.
    pub const VINDEP_SIGNATURES: u32 = 1 << 1;

    // Set if the plugin uses 629+ compatible structs. 629+ won't load without this.
    pub const VINDEP_STRUCTS_POST_629: u32 = 1 << 2;

    // Allows the plugin to load with all AE versions. Only set if you don't use structs
    // or check your version before accessing them manually.
    pub const VINDEPEX_NO_STRUCT_USE: u32 = 1 << 0;
}
