//!
//! @file sig.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Provides functions for defining and checking game code signatures.
//! @bug No known bugs.
//!

use std::vec::Vec;

use skse64::reloc::RelocAddr;

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
#[derive(Copy, Clone)]
pub struct Signature(&'static [Opcode]);

/// Helper to print a signature in the games code.
pub (in crate) struct BinarySig(RelocAddr, usize);

/// @brief Generates a new signature out of hex digits and question marks.
#[macro_export]
macro_rules! signature {
    ( $($sig:tt),+; $size:literal ) => {{
        let psize = [ $($crate::signature!(@munch $sig)),* ].len();
        if $size != psize {
            ::std::panic!("Patch size is incorrect.");
        }
        $crate::Signature::new(&[ $($crate::signature!(@munch $sig)),* ])
    }};

    ( @munch $op:literal ) => {
        $crate::Opcode::Code($op)
    };

    ( @munch ? ) => {
        $crate::Opcode::Any
    };
}

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
    pub (in crate) unsafe fn check(
        &self,
        a: usize
    ) -> Result<(), BinarySig> {
        assert!(a != 0);
        if self.0.len() == 0 { return Ok(()); }

        let mut diff = 0;
        skse64::safe::use_region(a, self.0.len(), || {
            for (i, op) in self.0.iter().enumerate() {
                if let Opcode::Code(b) = *op {
                    diff += if b == *(a as *mut u8).add(i) { 0 } else { 1 };
                }
            }
        });

        if diff > 0 {
            Err(BinarySig(RelocAddr::from_addr(a), self.len()))
        } else {
            Ok(())
        }
    }

    /// Checks how long the signature is.
    pub (in crate) fn len(
        &self
    ) -> usize {
        self.0.len()
    }
}

impl BinarySig {
    pub (in crate) fn reloc(
        &self
    ) -> RelocAddr {
        self.0
    }
}

impl std::fmt::Display for Signature {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>
    ) -> Result<(), std::fmt::Error> {
        write!(f, "{{ ")?;
        for op in self.0.iter() {
            if let Opcode::Code(b) = op {
                write!(f, "{:02x} ", b)?;
            } else {
                write!(f, "?? ")?;
            }
        }
        write!(f, "}}")
    }
}

impl std::fmt::Display for BinarySig {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>
    ) -> Result<(), std::fmt::Error> {
        // FIXME: This allocation really shouldn't be here.
        let mut sig: Vec<u8> = Vec::new();

        unsafe {
            // SAFETY: The caller of the diff function ensures this is a valid sig.
            skse64::safe::use_region(self.0.addr(), self.1, || {
                sig.extend_from_slice(
                    std::slice::from_raw_parts(self.0.addr() as *const u8, self.1)
                );
            });
        }

        write!(f, "{{ ")?;
        for b in sig.as_slice().iter() {
            write!(f, "{:02x} ", b)?;
        }
        write!(f, "}}")
    }
}
