//!
//! @file safe.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Provides functions for reading from/writing to the game code.
//! @bug No known bugs.
//!

use skse64::errors::skse_assert;

///
/// @brief Used to match code to pre-defined signatures.
///
/// This enumeration is used in the system that ensures that, regardless of game version, the
/// intended code is being overwritten.
///
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Opcode {
    Code(u8),
    Any
}

/// Identifies a distinct string of binary code within the skyrim binary.
pub struct Signature(&'static [Opcode]);

/// @brief Generates a new signature out of hex digits and question marks.
macro_rules! signature {
    ( $($sig:tt),+; $size:literal ) => {{
        let psize = [ $($crate::safe::signature!(@munch $sig)),* ].len();
        if $size != psize {
            ::std::panic!("Patch size is incorrect.");
        }
        $crate::safe::Signature::new(&[ $($crate::safe::signature!(@munch $sig)),* ])
    }};

    ( @munch $op:literal ) => {
        $crate::safe::Opcode::Code($op)
    };

    ( @munch ? ) => {
        $crate::safe::Opcode::Any
    };
}
pub (in crate) use signature;

impl Signature {
    /// Creates a new signature structure.
    pub const fn new(
        sig: &'static [Opcode]
    ) -> Self {
        Self(sig)
    }

    ///
    /// Checks the given signature against the given memory location.
    ///
    /// In order to use this function safely, the address range specified must be
    /// a valid part of the Skyrim binary.
    ///
    pub unsafe fn check(
        &self,
        a: usize
    ) -> Result<(), usize> {
        if self.0.len() == 0 { return Ok(()); }

        let mut diff = 0;
        skse64::safe::use_region(a, self.0.len(), || {
            let mut addr = a as *mut u8;
            skse_assert!(!addr.is_null());
            for op in self.0.iter() {
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

    /// Checks how long the signature is.
    pub fn len(
        &self
    ) -> usize {
        self.0.len()
    }
}

///
/// Uses the SKSE SafeWrite functions to set the given memory location.
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
        ::std::ptr::write_bytes::<u8>(a as *mut u8, c, n);
    });
}
