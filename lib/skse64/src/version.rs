//!
//! @file version.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes runtime version information and SKSE version structures.
//! @bug No known bugs.
//!

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
