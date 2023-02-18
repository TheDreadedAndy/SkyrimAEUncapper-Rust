//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat).
//! @brief Top level module file for SKSE64 reimplementation.
//! @bug No known bugs.
//!

pub use skse64_common::reloc;

pub mod version;
pub mod event;
mod errors;
pub mod log;
pub mod util;
pub mod plugin_api;
#[cfg(feature = "trampoline")] pub mod trampoline;
pub mod safe;
pub mod loader;

// For macros.
pub use core;
