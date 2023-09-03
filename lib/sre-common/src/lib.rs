//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Top level module for externel Skyrim RE headers.
//!

#![no_std]

pub mod versiondb;
pub mod skse64 {
    pub mod version;
    pub mod plugin_api;
    pub mod reloc;
}
