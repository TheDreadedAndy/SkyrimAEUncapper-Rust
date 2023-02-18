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

pub unsafe fn create(
    _t: Trampoline,
    _len: usize,
    _module: Option<NonNull<c_void>>
) {
    todo!();
}

pub unsafe fn write_jump6(
    _t: Trampoline,
    _src: usize,
    _dst: usize
) {
    todo!("Branch trampoline is not implemented");
}

pub unsafe fn write_call6(
    _t: Trampoline,
    _src: usize,
    _dst: usize
) {
    todo!("Branch trampoline is not implemented");
}

pub unsafe fn write_jump5(
    _t: Trampoline,
    _src: usize,
    _dst: usize
) {
    todo!("Branch trampoline is not implemented");
}

pub unsafe fn write_call5(
    _t: Trampoline,
    _src: usize,
    _dst: usize
) {
    todo!("Branch trampoline is not implemented");
}
