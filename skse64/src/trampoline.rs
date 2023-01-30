//!
//! @file trampoline.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes the global branch/local trampolines.
//! @bug No known bugs.
//!

use core::ffi::c_void;
use core::ptr::NonNull;

/// Encodes the trampoline which should be operated on.
#[repr(C)] pub enum Trampoline { Global, Local }

extern "system" {
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
