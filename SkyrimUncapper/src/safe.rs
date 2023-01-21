//!
//! @file safe.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Provides functions for reading from/writing to the game code.
//! @bug No known bugs.
//!

use std::ffi::{c_void, c_int};

use skse64::errors::skse_assert;

///
/// @brief Used to match code to pre-defined signatures.
///
/// This enumeration is used in the system that ensures that, regardless of game version, the
/// intended code is being overwritten.
///
#[derive(Copy, Clone)]
pub enum Opcode {
    Code(u8),
    Any
}

/// @brief Generates a new signature out of hex digits and question marks.
macro_rules! signature {
    ( $($sig:tt)* ) => {
        [ $crate::safe::signature!(@munch $($sig)*) ]
    };

    ( @munch $op:literal, $($sig:tt)* ) => {
        $crate::safe::Opcode::Code($op), $crate::safe::signature!(@munch $($sig)*)
    };

    ( @munch ?, $($sig:tt)* ) => {
        $crate::safe::Opcode::Any, $crate::safe::signature!(@munch $($sig)*)
    };

    ( @munch $op:literal ) => {
        $crate::safe::Opcode::Code($op)
    };

    ( @munch ? ) => {
        $crate::safe::Opcode::Any
    };
}

///
/// @brief Uses the SKSE SafeWrite functions to set the given memory location.
///
/// In order to use this function safely, the given address range must be a valid
/// part of the skyrim binary.
///
pub unsafe fn memset(
    a: usize,
    c: u8,
    n: usize
) {
    if n == 0 { return; }

    skse64::safe::use_region(a, n, || {
        libc::memset(a as *mut c_void, c as c_int, n);
    });
}

///
/// @brief Confirms that the given address contains the given code signature.
///
/// In order to use this function safely, the given address range must be a valid
/// part of the skyrim binary.
///
pub unsafe fn sigcheck(
    a: usize,
    sig: &[Opcode]
) -> Result<(), usize> {
    if sig.len() == 0 { return Ok(()); }

    let mut diff = 0;
    skse64::safe::use_region(a, sig.len(), || {
        let mut addr = a as *mut u8;
        skse_assert!(!addr.is_null());
        for op in sig.iter() {
            if let Opcode::Code(b) = *op {
                diff += if b == *addr { 0 } else { 1 };
            }
            addr = addr.add(1);
        }
    });

    if diff > 0 {
        Err(diff)
    } else {
        Ok(())
    }
}
