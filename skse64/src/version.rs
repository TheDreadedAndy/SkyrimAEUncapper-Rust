//!
//! @file version.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes various version constants and functions to manage them.
//! @bug No known bugs
//!
//! Bindgen can't evaluate macros, so these have to be written manually.
//!

use std::fmt::{Display, Formatter, Error};
use later::Later;

pub use crate::bind::{SKSE_VERSION_INTEGER, SKSE_VERSION_INTEGER_MINOR};
pub use crate::bind::{SKSE_VERSION_INTEGER_BETA, SKSE_VERSION_VERSTRING};
pub use crate::bind::{SKSE_VERSION_PADDEDSTRING, SKSE_VERSION_RELEASEIDX};
pub use crate::bind::{RUNTIME_TYPE_BETHESDA, RUNTIME_TYPE_GOG, RUNTIME_TYPE_EPIC};

/// @brief Wraps a skse version.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
#[repr(transparent)]
pub struct SkseVersion(u32);

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

/// Holds the running game/skse version. Initialized by the entry point.
pub (in crate) static RUNNING_GAME_VERSION: Later<SkseVersion> = Later::new();
pub (in crate) static RUNNING_SKSE_VERSION: Later<SkseVersion> = Later::new();

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

/// Gets the currently running game version.
pub fn current_runtime() -> SkseVersion {
    *RUNNING_GAME_VERSION
}

/// Gets the currently running SKSE version.
pub fn current_skse() -> SkseVersion {
    *RUNNING_SKSE_VERSION
}