//!
//! @file skse64.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Rust versions of SKSE definitions in skse_version.h, PluginAPI.h, and Relocation.h
//! @bug No known bugs.
//!

////////////////////////////////////////////////////////////////////////////////////////////////////
// Version information
////////////////////////////////////////////////////////////////////////////////////////////////////
// Additionally provides a wrapper type that mimics the original version creation macros.

pub mod version {
    use core::num::NonZeroU32;
    use core::fmt::{Display, Debug, Formatter, Error};

    /// @brief Wraps a skse version.
    #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct SkseVersion(NonZeroU32);

    pub const RUNTIME_TYPE_BETHESDA : u32 = 0;
    pub const RUNTIME_TYPE_GOG      : u32 = 1;
    pub const RUNTIME_TYPE_EPIC     : u32 = 2;

    pub const SAVE_FOLDER_NAME_BETHESDA : &'static str = "Skyrim Special Edition";
    pub const SAVE_FOLDER_NAME_GOG      : &'static str = "Skyrim Special Edition GOG";
    pub const SAVE_FOLDER_NAME_EPIC     : &'static str = "Skyrim Special Edition EPIC";

    pub const RUNTIME_VERSION_1_1_47: SkseVersion =
        SkseVersion::new(1, 1, 47, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_1_51: SkseVersion =
        SkseVersion::new(1, 1, 51, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_2_36: SkseVersion =
        SkseVersion::new(1, 2, 36, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_2_39: SkseVersion =
        SkseVersion::new(1, 2, 39, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_3_5: SkseVersion =
        SkseVersion::new(1, 3, 5, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_3_9: SkseVersion =
        SkseVersion::new(1, 3, 9, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_4_2: SkseVersion =
        SkseVersion::new(1, 4, 2, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_3: SkseVersion =
        SkseVersion::new(1, 5, 3, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_16: SkseVersion =
        SkseVersion::new(1, 5, 16, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_23: SkseVersion =
        SkseVersion::new(1, 5, 23, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_39: SkseVersion =
        SkseVersion::new(1, 5, 39, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_50: SkseVersion =
        SkseVersion::new(1, 5, 50, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_53: SkseVersion =
        SkseVersion::new(1, 5, 53, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_62: SkseVersion =
        SkseVersion::new(1, 5, 62, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_73: SkseVersion =
        SkseVersion::new(1, 5, 73, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_80: SkseVersion =
        SkseVersion::new(1, 5, 80, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_5_97: SkseVersion =
        SkseVersion::new(1, 5, 97, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_6_317: SkseVersion =
        SkseVersion::new(1, 6, 317, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_6_318: SkseVersion =
        SkseVersion::new(1, 6, 318, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_6_323: SkseVersion =
        SkseVersion::new(1, 6, 323, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_6_342: SkseVersion =
        SkseVersion::new(1, 6, 342, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_6_353: SkseVersion =
        SkseVersion::new(1, 6, 353, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_6_629: SkseVersion =
        SkseVersion::new(1, 6, 629, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_6_640: SkseVersion =
        SkseVersion::new(1, 6, 640, RUNTIME_TYPE_BETHESDA);
    pub const RUNTIME_VERSION_1_6_659_GOG: SkseVersion =
        SkseVersion::new(1, 6, 659, RUNTIME_TYPE_GOG);
    pub const RUNTIME_VERSION_1_6_678_EPIC: SkseVersion =
        SkseVersion::new(1, 6, 678, RUNTIME_TYPE_EPIC);

    pub const CURRENT_RELEASE_RUNTIME: SkseVersion = RUNTIME_VERSION_1_6_640;
    pub const PACKED_SKSE_VERSION: SkseVersion = SkseVersion::new(2, 2, 3, RUNTIME_TYPE_BETHESDA);

    impl SkseVersion {
        pub const fn new(
            major: u32,
            minor: u32,
            build: u32,
            sub: u32
        ) -> Self {
            Self::from_raw(
                (major << 24) |
                (minor << 16) |
                ((build & 0xFFF) << 4) |
                (sub & 0xF)
            )
        }

        /// @brief Converts a u32 to a skse version.
        pub const fn from_raw(
            v: u32
        ) -> Self {
            if let Some(v) = NonZeroU32::new(v) {
                Self(v)
            } else {
                panic!("Cannot create version 0.0.0.0!");
            }
        }

        /// @brief Gets the versions major revision.
        pub const fn major(
            &self
        ) -> u32 {
            self.0.get() >> 24
        }

        /// @brief Gets the versions minor revision.
        pub const fn minor(
            &self
        ) -> u32 {
            (self.0.get() >> 16) & 0xFF
        }

        /// @brief Gets the versions build number.
        pub const fn build(
            &self
        ) -> u32 {
            (self.0.get() >> 4) & 0xFFF
        }

        /// @brief Gets the versions runtime type.
        pub const fn runtime_type(
            &self
        ) -> u32 {
            self.0.get() & 0xF
        }

        /// Gets the save folder name for this version.
        pub const fn save_folder(
            &self
        ) -> &'static str {
            match self.runtime_type() {
                RUNTIME_TYPE_GOG  => SAVE_FOLDER_NAME_GOG,
                RUNTIME_TYPE_EPIC => SAVE_FOLDER_NAME_EPIC,
                _                 => SAVE_FOLDER_NAME_BETHESDA
            }
        }
    }

    impl Display for SkseVersion {
        fn fmt(
            &self,
            f: &mut Formatter<'_>
        ) -> Result<(), Error> {
            let runtime = match self.runtime_type() {
                RUNTIME_TYPE_BETHESDA => "Bethesda",
                RUNTIME_TYPE_GOG => "GOG",
                RUNTIME_TYPE_EPIC => "Epic",
                _ => "Unknown"
            };

            write!(f, "{}.{}.{} ({})", self.major(), self.minor(), self.build(), runtime)
        }
    }

    impl Debug for SkseVersion {
        fn fmt(
            &self,
            f: &mut Formatter<'_>
        ) -> Result<(), Error> {
            write!(f, "{}.{}.{}.{}", self.major(), self.minor(), self.build(), self.runtime_type())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Relocation
////////////////////////////////////////////////////////////////////////////////////////////////////
// Note that this definition has been extended to allow this type to work outside of the skyrim
// runtime by defaulting the base address to 0x140000000.

pub mod reloc {
    use core_util::Later;

    /// Holds a game address, which can be accessed by offset or address.
    #[repr(transparent)]
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
    pub struct RelocAddr(usize);

    /// Holds the base address of the skyrim binary.
    static BASE_ADDR: Later<usize> = Later::new();

    impl RelocAddr {
        /// Initializes the relocation manager with the given base address.
        ///
        /// This function may only be called once.
        pub fn init_manager(
            addr: usize
        ) {
            BASE_ADDR.init(addr);
        }

        /// Gets the base address of the skyrim binary.
        pub fn base() -> usize {
            *BASE_ADDR
        }

        /// Creates a reloc addr from an offset.
        pub const fn from_offset(
            offset: usize
        ) -> Self {
            Self(offset)
        }

        /// Creates a reloc addr from an address.
        pub fn from_addr(
            addr: usize
        ) -> Self {
            assert!(Self::base() <= addr);
            Self(addr - Self::base())
        }

        /// Gets the underlying offset of the RelocAddr.
        pub const fn offset(
            self
        ) -> usize {
            self.0
        }

        /// Gets the actual address of the RelocAddr.
        pub fn addr(
            self
        ) -> usize {
            Self::base() + self.0
        }
    }

    impl core::ops::Add<usize> for RelocAddr {
        type Output = Self;
        fn add(
            self,
            rhs: usize
        ) -> Self::Output {
            Self(self.0 + rhs)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Plugin API
////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod plugin_api {
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

    /// See SKSE notes. The functions may only be called during specific phases.
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

    ////////////////////////////////////////////////////////////////////////////////////////////////

    impl PluginInfo {
        pub const VERSION: u32 = 1;
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
}
