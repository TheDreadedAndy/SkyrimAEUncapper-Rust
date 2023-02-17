//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat).
//! @brief Top level module file for SKSE64 reimplementation.
//! @bug No known bugs.
//!

pub mod version;
pub mod util;

#[cfg(not(feature = "not_plugin"))] pub mod query;
#[cfg(not(feature = "not_plugin"))] pub mod event;
#[cfg(not(feature = "not_plugin"))] mod errors;
#[cfg(not(feature = "not_plugin"))] pub mod log;
#[cfg(not(feature = "not_plugin"))] pub mod reloc;
#[cfg(not(feature = "not_plugin"))] pub mod plugin_api;
#[cfg(all(feature = "trampoline", not(feature = "not_plugin")))] pub mod trampoline;
#[cfg(not(feature = "not_plugin"))] pub mod safe;
#[cfg(not(feature = "not_plugin"))] pub mod loader;

// For macros.
pub use core;
