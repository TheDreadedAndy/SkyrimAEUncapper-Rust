//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat).
//! @brief Top level module file for SKSE FFI.
//! @bug No known bugs.
//!

mod bind;

// For macros.
pub use core;

///
/// @brief Exposes the various version constants and functions to manage them.
///
/// Bindgen can't evaluate macros, so these have to be written manually.
///
pub mod version {
    use std::fmt::{Display, Formatter, Error};

    pub use crate::bind::{SKSE_VERSION_INTEGER, SKSE_VERSION_INTEGER_MINOR};
    pub use crate::bind::{SKSE_VERSION_INTEGER_BETA, SKSE_VERSION_VERSTRING};
    pub use crate::bind::{SKSE_VERSION_PADDEDSTRING, SKSE_VERSION_RELEASEIDX};
    pub use crate::bind::{RUNTIME_TYPE_BETHESDA, RUNTIME_TYPE_GOG, RUNTIME_TYPE_EPIC};

    /// @brief Wraps a skse version.
    #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct SkseVersion(u32);

    impl SkseVersion {
        const fn new(
            major: u32,
            minor: u32,
            build: u32,
            sub: u32
        ) -> Self {
            Self(
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
            Self(v)
        }

        /// @brief Gets the versions major revision.
        pub const fn major(
            &self
        ) -> u32 {
            self.0 >> 24
        }

        /// @brief Gets the versions minor revision.
        pub const fn minor(
            &self
        ) -> u32 {
            (self.0 >> 16) & 0xFF
        }

        /// @brief Gets the versions build number.
        pub const fn build(
            &self
        ) -> u32 {
            (self.0 >> 4) & 0xFFF
        }

        /// @brief Gets the versions runtime type.
        pub const fn runtime_type(
            &self
        ) -> u32 {
            self.0 & 0xF
        }
    }

    /// @brief Allows a skse64 version to be printed.
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
    pub const PACKED_SKSE_VERSION: SkseVersion = SkseVersion::new(
        SKSE_VERSION_INTEGER,
        SKSE_VERSION_INTEGER_MINOR,
        SKSE_VERSION_INTEGER_BETA,
        RUNTIME_TYPE_BETHESDA
    );
}

pub mod errors {
    use core::ffi::{c_ulong, c_char};
    pub use ctypes::cstr;

    extern "C" {
        /// @brief SKSE panic function.
        #[link_name = "SKSE64_Errors__assert_failed__"]
        pub fn skse_panic_impl(file: *const c_char, line: c_ulong, msg: *const c_char) -> !;
    }

    /// @brief Uses the SKSE panic handler to terminate the application.
    #[macro_export]
    macro_rules! skse_halt {
        ( $s:expr ) => {{
            let s = $crate::errors::cstr!($s);
            let file = $crate::errors::cstr!($crate::core::file!());
            let line = $crate::core::line!();

            unsafe {
                $crate::errors::skse_panic_impl(file, line as $crate::core::ffi::c_ulong, s);
            }
        }};
    }

    /// @brief Uses the SKSE panic handler to assert a condition.
    #[macro_export]
    macro_rules! skse_assert {
        ( $cond:expr ) => {
            if !($cond) {
                $crate::skse_halt!($crate::core::stringify!($cond));
            }
        };

        ( $cond:expr, $lit:expr ) => {
            if !($cond) {
                $crate::skse_halt!($lit);
            }
        };
    }

    pub use skse_assert;
    pub use skse_halt;
}

/// @brief Wraps the SKSE logging API.
pub mod log {
    use std::ffi::{c_char, CString};
    use std::path::Path;

    extern "C" {
        #[link_name = "SKSE64_DebugLog__open__"]
        fn glog_open(path: *const c_char);

        #[link_name = "SKSE64_DebugLog__message__"]
        fn glog_message(msg: *const c_char);

        #[link_name = "SKSE64_DebugLog__error__"]
        fn glog_error(msg: *const c_char);
    }

    /// @brief Opens the file pointed to by the path as the log file for the plugin.
    pub fn open(
        path: &Path
    ) {
        unsafe {
            // SAFETY: We are giving this function a valid C string.
            glog_open(CString::new(path.to_str().unwrap()).unwrap().as_c_str().as_ptr());
        }
    }

    #[doc(hidden)]
    pub fn skse_message_impl(
        msg: &str
    ) {
        unsafe {
            // SAFETY: we are giving this fn a valid string.
            glog_message(CString::new(msg).unwrap().as_c_str().as_ptr());
        }
    }

    #[doc(hidden)]
    pub fn skse_error_impl(
        msg: &str
    ) {
        unsafe {
            // SAFETY: We are giving this fn a valid string.
            glog_error(CString::new(msg).unwrap().as_c_str().as_ptr());
        }
    }

    #[macro_export]
    macro_rules! skse_message {
        ( $($fmt:tt)* ) => {
            $crate::log::skse_message_impl(::std::format!($($fmt)*).as_str());
        };
    }

    #[macro_export]
    macro_rules! skse_error {
        ( $($fmt:tt)* ) => {
            $crate::log::skse_error_impl(::std::format!($($fmt)*).as_str());
        };
    }

    pub use skse_message;
    pub use skse_error;
}

/// @brief Exposes the get_runtime_dir() function.
pub mod utilities {
    pub use std::ffi::{c_char, CStr, CString};
    pub use std::path::PathBuf;

    extern "C" {
        fn SKSE64_Utilities__get_runtime_dir__() -> *const c_char;
    }

    pub fn get_runtime_dir() -> PathBuf {
        PathBuf::from(unsafe {
            // SAFETY: We ensure that we don't call any other functions which touch the
            //         underlying std::string for the runtime dir until we have copied it.
            CStr::from_ptr(SKSE64_Utilities__get_runtime_dir__())
        }.to_owned().into_string().unwrap())
    }
}

/// @brief Exposes the plugin API data structure.
pub mod plugin_api {
    pub use crate::bind::SKSEInterface;
    pub use crate::bind::SKSEPluginVersionData;
    pub use crate::bind::SKSEPluginVersionData_kVersion;
    pub use crate::bind::SKSEPluginVersionData_kVersionIndependent_AddressLibraryPostAE;
    pub use crate::bind::SKSEPluginVersionData_kVersionIndependent_Signatures;
    pub use crate::bind::SKSEPluginVersionData_kVersionIndependent_StructsPost629;
    pub use crate::bind::SKSEPluginVersionData_kVersionIndependentEx_NoStructUse;
}

/// @brief Exposes the global branch/local trampolines.
pub mod trampoline {
    use core::ffi::c_void;
    use core::ptr::NonNull;

    /// @brief Encodes the trampoline which should be operated on.
    #[repr(C)] pub enum Trampoline { Global, Local }

    extern "C" {
        // module == None => "use skyrim module".
        #[link_name = "SKSE64_BranchTrampoline__create__"]
        pub fn create(t: Trampoline, len: usize, module: Option<NonNull<c_void>>);

        #[link_name = "SKSE64_BranchTrampoline__destroy__"]
        pub fn destroy(t: Trampoline);

        #[link_name = "SKSE64_BranchTrampoline__write_jump6__"]
        pub fn write_jump6(t: Trampoline, src: usize, dst: usize);

        #[link_name = "SKSE64_BranchTrampoline__write_call6__"]
        pub fn write_call6(t: Trampoline, src: usize, dst: usize);

        #[link_name = "SKSE64_BranchTrampoline__write_jump5__"]
        pub fn write_jump5(t: Trampoline, src: usize, dst: usize);

        #[link_name = "SKSE64_BranchTrampoline__write_call5__"]
        pub fn write_call5(t: Trampoline, src: usize, dst: usize);
    }
}

/// @brief Exposes the safe-write functions.
pub mod safe {
    use core::ffi::c_int;

    extern "C" {
        fn SKSE64_SafeWrite__virtual_protect__(
            addr: usize,
            size: usize,
            new_prot: u32,
            old_prot: *mut u32
        );
        fn SKSE64_SafeWrite__safe_write_jump__(src: usize, dst: usize) -> c_int;
        fn SKSE64_SafeWrite__safe_write_call__(src: usize, dst: usize) -> c_int;
    }

    /// @brief Temporarily marks the given memory region for read/write, then call the given fn.
    pub unsafe fn use_region(
        addr: usize,
        size: usize,
        func: impl FnOnce()
    ) {
        const PAGE_EXECUTE_READWRITE: u32 = 0x40;
        let mut old_prot: u32 = 0;
        SKSE64_SafeWrite__virtual_protect__(addr, size, PAGE_EXECUTE_READWRITE, &mut old_prot);
        func();
        SKSE64_SafeWrite__virtual_protect__(addr, size, old_prot, &mut old_prot);
    }

    /// @brief Writes a 5-byte jump to the given address.
    pub unsafe fn safe_write_jump(
        src: usize,
        dst: usize
    ) -> Result<(), ()> {
        if SKSE64_SafeWrite__safe_write_jump__(src, dst) >= 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// @brief Writes a 5-byte call to the given address.
    pub unsafe fn safe_write_call(
        src: usize,
        dst: usize
    ) -> Result<(), ()> {
        if SKSE64_SafeWrite__safe_write_call__(src, dst) >= 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}
